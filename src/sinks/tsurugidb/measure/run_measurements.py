#!/usr/bin/env python3
"""
TsurugiDB Sink 性能測定実行スクリプト

- src/sinks/tsurugidb/measure/plan.md に沿って、一連の測定シナリオを実行する。
- 初回は列数ごとのダミー CSV ログを生成し、その後は再利用する。
- Vector は、--vector-bin と --vector-config が指定されていれば実際に起動し、
  指定されていなければドライランとして実行コマンドのみ表示する。

前提（例）:
- Vector の設定ファイル (例: measure/vector_stdin_tsurugi.toml) では、
  環境変数 TSURUGI_TABLE, VECTOR_IN_FLIGHT_LIMIT などを参照できるようにしておく。
  例（イメージ）:

    [sources.my_source]
    type = "stdin"

    [sinks.my_tsurugi_sink]
    type = "tsurugidb"
    endpoint = "tcp://localhost:12345"
    table = "${TSURUGI_TABLE}"

    [sinks.my_tsurugi_sink.request]
    in_flight_limit = ${VECTOR_IN_FLIGHT_LIMIT}

※ 上記の環境変数展開方法は利用する Vector のバージョンや設定仕様に合わせて調整してください。
"""

import argparse
import csv
import os
import random
import subprocess
import sys
import time
from dataclasses import dataclass
from datetime import datetime, timedelta
from pathlib import Path
from typing import List, Optional, Dict, Any


# ------------------------------
# 設定値（必要に応じて調整）
# ------------------------------

# 列数バリエーション
COLUMN_COUNTS = [1, 10, 100]

# ベースとなるイベント発生速度候補（events/sec）
#DEFAULT_EVENT_RATES = [10_000, 50_000, 100_000, 200_000]
DEFAULT_EVENT_RATES = [100,1000,10000]

# 並列度候補
# DEFAULT_PARALLELISMS = [1,2,4,8,16,32,64]
DEFAULT_PARALLELISMS = [4,8,16,32]

# 各測定のデフォルト実行時間（秒）
# DEFAULT_DURATION_SEC = 120
DEFAULT_DURATION_SEC = 10

# ダミーログ生成時に確保しておく最大イベント数
# （最大のレート × 最大の想定計測時間）より少し余裕を持たせる
# MAX_EVENTS_PER_FILE = 300_000 * 300  # 例: 200k * 300sec より少し大きめ
MAX_EVENTS_PER_FILE = 10000 * 10  # 例: 200k * 300sec より少し大きめ

# ------------------------------
# データ構造
# ------------------------------


@dataclass
class Scenario:
    name: str
    columns: int
    event_rate: int  # events/sec
    parallelism: int
    duration_sec: int


# ------------------------------
# ダミー CSV ログ生成
# ------------------------------


def generate_dummy_csv(path: Path, num_events: int, num_double_cols: int) -> None:
    """指定パスにダミー CSV を生成する。

    1列目: ISO8601 タイムスタンプ文字列
    2列目以降: DOUBLE 相当のランダム値
    """
    path.parent.mkdir(parents=True, exist_ok=True)
    print(f"[INFO] Generating dummy CSV: {path} (events={num_events}, cols={num_double_cols})")

    start_time = datetime(2025, 1, 1, 0, 0, 0)
    delta = timedelta(microseconds=100)  # 行ごとに 0.0001 秒ずつ進める

    with path.open("w", newline="") as f:
        writer = csv.writer(f)
        current_time = start_time
        for _ in range(num_events):
            ts_str = current_time.isoformat() + "Z"
            # DOUBLE 列をランダム生成
            doubles = [f"{random.random() * 1000:.6f}" for _ in range(num_double_cols)]
            writer.writerow([ts_str] + doubles)
            current_time += delta

    print(f"[INFO] Dummy CSV generated: {path}")


def ensure_dummy_logs(base_dir: Path, max_events: int) -> Dict[int, Path]:
    """列数ごとのダミー CSV ログを生成または再利用する。

    戻り値: {列数: パス}
    """
    data_dir = base_dir / "data"
    data_dir.mkdir(parents=True, exist_ok=True)

    result: Dict[int, Path] = {}
    for cols in COLUMN_COUNTS:
        # 1 列構成: timestamp + DOUBLE 1列分
        num_double_cols = cols
        csv_path = data_dir / f"log_events_{cols}cols.csv"
        if not csv_path.exists():
            generate_dummy_csv(csv_path, max_events, num_double_cols)
        else:
            print(f"[INFO] Reusing existing CSV: {csv_path}")
        result[cols] = csv_path

    return result


# ------------------------------
# シナリオ定義
# ------------------------------


def build_scenarios() -> List[Scenario]:
    scenarios: List[Scenario] = []

    # 5.1 ベースライン測定
    scenarios.append(
        Scenario(
            name="baseline_cols1_p1_r10k",
            columns=1,
            parallelism=1,
            event_rate=10_000,
            duration_sec=DEFAULT_DURATION_SEC,
        )
    )

    # 5.2 イベント発生速度スケーリング（列数=10, 並列=4）
    for rate in DEFAULT_EVENT_RATES:
        scenarios.append(
            Scenario(
                name=f"rate_scaling_cols10_p4_r{rate}",
                columns=10,
                parallelism=4,
                event_rate=rate,
                duration_sec=DEFAULT_DURATION_SEC,
            )
        )

    # 5.3 Vector 並列数スケーリング（列数=10, レート=100k）
    fixed_rate = 100_000
    for p in DEFAULT_PARALLELISMS:
        scenarios.append(
            Scenario(
                name=f"parallel_scaling_cols10_p{p}_r{fixed_rate}",
                columns=10,
                parallelism=p,
                event_rate=fixed_rate,
                duration_sec=DEFAULT_DURATION_SEC,
            )
        )

    # 5.4 列数スケーリング（列数=1,10,100,1000, 並列=4, レート=100k）
    for cols in COLUMN_COUNTS:
        scenarios.append(
            Scenario(
                name=f"column_scaling_cols{cols}_p4_r{fixed_rate}",
                columns=cols,
                parallelism=4,
                event_rate=fixed_rate,
                duration_sec=DEFAULT_DURATION_SEC,
            )
        )

    return scenarios


# ------------------------------
# Vector 実行ロジック
# ------------------------------


def run_vector_with_rate_control(
    vector_bin: Optional[str],
    vector_config: Optional[Path],
    csv_path: Path,
    scenario: Scenario,
    table_prefix: str,
    extra_env: Optional[Dict[str, str]] = None,
    dry_run: bool = False,
) -> Dict[str, Any]:
    """CSV を元に、指定レートで行を stdin 経由で Vector に流し込む。

    - Vector 側は stdin source を前提とする。
    - table 名や in_flight_limit は環境変数で指定する想定。

    戻り値: 実行結果のメタ情報（実測レートなど）
    """
    num_events = scenario.event_rate * scenario.duration_sec
    table_name = f"{table_prefix}{scenario.columns}"

    env = os.environ.copy()
    env["TSURUGI_TABLE"] = table_name
    env["VECTOR_IN_FLIGHT_LIMIT"] = str(scenario.parallelism)
    env["MEASURE_SCENARIO_NAME"] = scenario.name
    if extra_env:
        env.update(extra_env)

    cmd: List[str] = []
    if vector_bin and vector_config:
        cmd = [vector_bin, "-c", str(vector_config)]
    else:
        # vector を実際には起動しない（ドライラン）
        dry_run = True

    print("\n============================================")
    print(f"[SCENARIO] {scenario.name}")
    print(f"  columns      : {scenario.columns}")
    print(f"  event_rate   : {scenario.event_rate} events/sec")
    print(f"  parallelism  : {scenario.parallelism}")
    print(f"  duration     : {scenario.duration_sec} sec")
    print(f"  csv_path     : {csv_path}")
    print(f"  table_name   : {table_name}")
    if cmd:
        print(f"  vector_cmd   : {' '.join(cmd)}")
    else:
        print("  vector_cmd   : (not specified; dry-run mode)")
    print("============================================")

    if dry_run:
        return {
            "scenario": scenario.name,
            "columns": scenario.columns,
            "event_rate": scenario.event_rate,
            "parallelism": scenario.parallelism,
            "duration_sec": scenario.duration_sec,
            "csv_path": str(csv_path),
            "table_name": table_name,
            "status": "dry_run",
        }

    # Vector プロセス起動
    proc = subprocess.Popen(
        cmd,
        stdin=subprocess.PIPE,
        stdout=sys.stdout,
        stderr=sys.stderr,
        env=env,
        text=True,  # stdin/stdout をテキストモードに
        bufsize=1,
    )

    sent = 0
    start_time = time.perf_counter()
    interval = 1.0 / scenario.event_rate
    next_time = start_time

    try:
        with csv_path.open("r") as f:
            for line in f:
                if sent >= num_events:
                    break

                # 行をそのまま stdin に書き込む
                assert proc.stdin is not None
                proc.stdin.write(line)
                sent += 1

                # レート制御
                next_time += interval
                sleep_time = next_time - time.perf_counter()
                if sleep_time > 0:
                    time.sleep(sleep_time)

        # 入力完了を通知
        if proc.stdin:
            proc.stdin.close()

        # プロセス終了待ち
        returncode = proc.wait()
    except KeyboardInterrupt:
        print("[WARN] Interrupted by user, terminating Vector process...")
        proc.terminate()
        returncode = proc.wait()
    except Exception as e:  # noqa: BLE001
        print(f"[ERROR] Exception during scenario {scenario.name}: {e}")
        proc.terminate()
        returncode = proc.wait()

    end_time = time.perf_counter()
    elapsed = end_time - start_time
    actual_rate = sent / elapsed if elapsed > 0 else 0.0

    print(f"[RESULT] scenario={scenario.name}")
    print(f"  sent_events  : {sent}")
    print(f"  elapsed_sec  : {elapsed:.3f}")
    print(f"  actual_rate  : {actual_rate:.1f} events/sec")
    print(f"  returncode   : {returncode}")

    status = "ok" if returncode == 0 else "error"

    return {
        "scenario": scenario.name,
        "columns": scenario.columns,
        "event_rate": scenario.event_rate,
        "parallelism": scenario.parallelism,
        "duration_sec": scenario.duration_sec,
        "csv_path": str(csv_path),
        "table_name": table_name,
        "sent_events": sent,
        "elapsed_sec": elapsed,
        "actual_rate": actual_rate,
        "returncode": returncode,
        "status": status,
    }


# ------------------------------
# メイン
# ------------------------------


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="TsurugiDB Sink 性能測定実行スクリプト (measure/plan.md 準拠)",
    )
    parser.add_argument(
        "--base-dir",
        type=Path,
        default=Path(__file__).resolve().parent,
        help="測定用ディレクトリ (デフォルト: このスクリプトと同じディレクトリ)",
    )
    parser.add_argument(
        "--vector-bin",
        type=str,
        default=None,
        help="vector 実行バイナリへのパス (指定しない場合はドライラン)",
    )
    parser.add_argument(
        "--vector-config",
        type=Path,
        default=None,
        help="vector 設定ファイルへのパス (stdin source + tsurugidb sink を想定)",
    )
    parser.add_argument(
        "--table-prefix",
        type=str,
        default="log_events_",
        help="テーブル名のプレフィックス (例: log_events_ → log_events_1, log_events_10, ...)",
    )
    parser.add_argument(
        "--max-events-per-file",
        type=int,
        default=MAX_EVENTS_PER_FILE,
        help="ダミー CSV 1 ファイルあたりの最大イベント数",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="vector を起動せず、シナリオ情報と想定コマンドを表示するのみ",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    base_dir: Path = args.base_dir
    base_dir.mkdir(parents=True, exist_ok=True)

    # 測定開始日時（結果ファイル名および各行に埋め込む）
    measure_start = datetime.now()
    measure_start_str = measure_start.strftime("%Y%m%d-%H%M%S")

    # 1. ダミーログ生成（初回のみ）
    csv_paths = ensure_dummy_logs(base_dir, args.max_events_per_file)

    # 2. シナリオ一覧を構築
    scenarios = build_scenarios()

    # 3. 各シナリオを順番に実行
    all_results: List[Dict[str, Any]] = []
    for scenario in scenarios:
        csv_path = csv_paths[scenario.columns]
        result = run_vector_with_rate_control(
            vector_bin=args.vector_bin,
            vector_config=args.vector_config,
            csv_path=csv_path,
            scenario=scenario,
            table_prefix=args.table_prefix,
            extra_env=None,
            dry_run=args.dry_run,
        )
        all_results.append(result)

    # 4. 測定結果を CSV ファイルとして出力
    data_dir = base_dir / "data"
    data_dir.mkdir(parents=True, exist_ok=True)
    result_csv_path = data_dir / f"result_{measure_start_str}.csv"

    with result_csv_path.open("w", newline="") as f:
        writer = csv.writer(f)
        # 1 行目はヘッダーとし、2 行目以降を各シナリオ 1 行で出力する
        writer.writerow(
            [
                "measure_start",  # 測定開始日時（ISO8601）
                "scenario",
                "columns",
                "event_rate",
                "parallelism",
                "duration_sec",
                "sent_events",
                "elapsed_sec",
                "actual_rate",
                "status",
                "returncode",
                "table_name",
            ]
        )
        for r in all_results:
            writer.writerow(
                [
                    measure_start.isoformat(),
                    r.get("scenario", ""),
                    r.get("columns", ""),
                    r.get("event_rate", ""),
                    r.get("parallelism", ""),
                    r.get("duration_sec", ""),
                    r.get("sent_events", 0),
                    f"{r.get('elapsed_sec', 0.0):.6f}",
                    f"{r.get('actual_rate', 0.0):.1f}",
                    r.get("status", ""),
                    r.get("returncode", ""),
                    r.get("table_name", ""),
                ]
            )

    # 5. 簡易的なサマリを標準出力に表示
    print("\n===== SUMMARY =====")
    print(f"[INFO] Result CSV: {result_csv_path}")
    for r in all_results:
        print(
            f"{r['scenario']}: status={r['status']}, "
            f"cols={r['columns']}, rate={r['event_rate']}, "
            f"parallel={r['parallelism']}, "
            f"actual_rate={r.get('actual_rate', 0):.1f}",
        )


if __name__ == "__main__":
    main()
