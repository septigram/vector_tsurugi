# TsurugiDB Sink 結合テスト実施手順書

**作成日**: 2025年12月15日  
**対象コンポーネント**: Vector TsurugiDB Sink

## 目次

1. [概要](#概要)
2. [前提条件](#前提条件)
3. [環境準備](#環境準備)
4. [テスト実行方法](#テスト実行方法)
5. [テスト内容](#テスト内容)
6. [トラブルシューティング](#トラブルシューティング)
7. [参考情報](#参考情報)

---

## 概要

このドキュメントは、VectorプロジェクトのTsurugiDB Sinkの結合テストを実施するための手順書です。

結合テストは、Vectorが実際のTsurugiDBサーバーと正しく動作することを確認するためのテストです。

### テストの特徴

- **実際のTsurugiDBサーバーが必要**: テストを実行する前に、TsurugiDBサーバーが起動している必要があります
- **テストは `#[ignore]` 属性が付いているため、明示的に実行する必要がある**: `--ignored` フラグを使用して実行します
- **環境変数 `TSURUGI_ENDPOINT` でTsurugiDBサーバーのエンドポイントを指定**: デフォルトは `tcp://localhost:12345`

### よくあるエラー

テスト実行時に以下のエラーが発生する場合、TsurugiDBサーバーが起動していないか、接続できない状態です：

- `Failed to connect to Tsurugi: Wire::pull() slot=0 timeout` - 接続タイムアウト
- `Failed to connect to Tsurugi: Connection refused` - 接続拒否

これらのエラーが発生した場合、[トラブルシューティング](#トラブルシューティング)セクションを参照してください。

---

## 前提条件

### 必須ソフトウェア

以下のソフトウェアがインストールされている必要があります：

1. **Rustツールチェーン**
   - Rust 1.88以上
   - `cargo`コマンドが使用可能であること

2. **C/C++ビルドツール**
   - `gcc` または `clang`
   - `libclang-dev` または `clang` パッケージ（`bindgen`が必要とするため）
   - `build-essential` パッケージ（推奨）

3. **TsurugiDBサーバー**
   - TsurugiDBサーバーが起動していること
   - デフォルトエンドポイント: `tcp://localhost:12345`
   - または環境変数 `TSURUGI_ENDPOINT` で指定

4. **その他のツール**
   - `make`（オプション）

### 環境変数

以下の環境変数を設定できます：

- **`TSURUGI_ENDPOINT`**: TsurugiDBサーバーへの接続エンドポイント
  - デフォルト値: `tcp://localhost:12345`
  - 形式: `tcp://host:port`

例：
```bash
export TSURUGI_ENDPOINT="tcp://localhost:12345"
```

---

## 環境準備

### 1. ビルドツールの確認

必要なビルドツールがインストールされていることを確認します：

```bash
# gccの確認
gcc --version

# libclang-devの確認（Ubuntu/Debianの場合）
dpkg -l | grep libclang-dev

# インストールされていない場合（Ubuntu/Debian）
sudo apt-get update
sudo apt-get install -y libclang-dev build-essential
```

### 2. プロジェクトのビルド

プロジェクトが正常にビルドできることを確認します：

```bash
cd /path/to/vector_tsurugi
cargo build --features sinks-tsurugidb
```

**注意**: ビルド時に `stddef.h` が見つからないというエラーが発生する場合：
- `libclang-dev` パッケージがインストールされているか確認
- `clang` コマンドがインストールされているか確認（`which clang`）
- 必要に応じて `sudo apt-get install -y libclang-dev clang` を実行

### 3. TsurugiDBサーバーの起動

TsurugiDBサーバーが起動していることを確認します：

```bash
# TsurugiDBサーバーの起動方法は、TsurugiDBのドキュメントを参照してください
# デフォルトでは localhost:12345 でリッスンしている必要があります
```

### 4. 接続確認（オプション）

TsurugiDBサーバーへの接続を確認するには、TsurugiDBのクライアントツールを使用します。

---

## テスト実行方法

### 方法1: cargo test を使用（推奨）

**重要**: 統合テストを実行するには、`sinks-tsurugidb` と `tsurugidb_sink-integration-tests` の両方のフィーチャーを有効にする必要があります。

#### すべての結合テストを実行

```bash
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --ignored
```

#### 特定のテストを実行

```bash
# healthcheck_passes テストを実行
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::healthcheck_passes -- --ignored

# insert_single_event テストを実行
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insert_single_event -- --ignored

# insert_multiple_events テストを実行
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insert_multiple_events -- --ignored

# insertion_fails_missing_table テストを実行
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insertion_fails_missing_table -- --ignored
```

#### 環境変数を指定して実行

```bash
TSURUGI_ENDPOINT="tcp://your-host:12345" \
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --ignored
```

### 方法2: すべてのテスト（ignoredを含む）を実行

```bash
# ignoredが付いていないテストも含めてすべて実行
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests
```

### 方法3: 詳細なログ出力と共に実行

```bash
RUST_BACKTRACE=full \
VECTOR_LOG=vector=debug \
TSURUGI_ENDPOINT="tcp://localhost:12345" \
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --ignored --nocapture
```

### 注意事項

- テストは `#[ignore]` 属性が付いているため、`--ignored` フラグが必要です
- 実際のTsurugiDBサーバーが起動している必要があります
- テストは自動的にテーブルを作成しますが、事前にデータベースが存在している必要があります

---

## テスト内容

### 実装されているテスト

以下のテストが `src/sinks/tsurugidb/integration_tests.rs` に実装されています：

#### 1. `healthcheck_passes`

- **目的**: TsurugiDBサーバーへの接続が正常に確立できることを確認
- **前提条件**: TsurugiDBサーバーが起動していること
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::healthcheck_passes -- --ignored
  ```

#### 2. `healthcheck_fails_unknown_host`

- **目的**: 存在しないホストへの接続が適切に失敗することを確認
- **前提条件**: なし（モックテスト）
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::healthcheck_fails_unknown_host
  ```
- **注意**: このテストは `#[ignore]` が付いていないため、通常のテスト実行でも実行されますが、現在は `build` 時に接続を試みるため、接続エラーで失敗する可能性があります

#### 3. `healthcheck_fails_timed_out`

- **目的**: タイムアウトが適切に処理されることを確認
- **前提条件**: なし（モックテスト）
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::healthcheck_fails_timed_out
  ```
- **注意**: このテストは `#[ignore]` が付いていないため、通常のテスト実行でも実行されますが、現在は `build` 時に接続を試みるため、接続エラーで失敗する可能性があります

#### 4. `insert_single_event`

- **目的**: 単一のイベントをTsurugiDBに挿入できることを確認
- **前提条件**: 
  - TsurugiDBサーバーが起動していること
  - テスト用のテーブルが作成されること（テスト内で自動作成）
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insert_single_event -- --ignored
  ```

#### 5. `insert_multiple_events`

- **目的**: 複数のイベントをTsurugiDBに挿入できることを確認
- **前提条件**: 
  - TsurugiDBサーバーが起動していること
  - テスト用のテーブルが作成されること（テスト内で自動作成）
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insert_multiple_events -- --ignored
  ```

#### 6. `insertion_fails_missing_table`

- **目的**: 存在しないテーブルへの挿入が適切に失敗することを確認
- **前提条件**: TsurugiDBサーバーが起動していること
- **実行方法**:
  ```bash
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insertion_fails_missing_table -- --ignored
  ```

### テストで使用されるテーブル構造

テストは以下の構造のテーブルを自動的に作成します：

```sql
CREATE TABLE <table_name> (
    id BIGINT,
    host TEXT,
    event_timestamp TIMESTAMP,
    message TEXT,
    payload JSON
)
```

**注意**: 
- `timestamp`はTsurugiDBの予約語の可能性があるため、`event_timestamp`を使用しています
- `TIMESTAMPTZ`の代わりに`TIMESTAMP`を使用しています（TsurugiDBのサポートを確認）

テーブル名は `random_table_name()` 関数で生成されるランダムな名前が使用されます。

---

## トラブルシューティング

### よくある問題と解決方法

#### 1. ビルドエラー: "stddef.h file not found"

**問題**: プロジェクトのビルド時に `stddef.h` が見つからないエラーが発生する

**解決方法**:
- `libclang-dev` パッケージをインストール（Ubuntu/Debianの場合）
  ```bash
  sudo apt-get update
  sudo apt-get install -y libclang-dev
  ```
- `clang` コマンドをインストール（オプション）
  ```bash
  sudo apt-get install -y clang
  ```
- 環境変数を設定してビルドを試行
  ```bash
  LIBCLANG_PATH=/usr/lib/llvm-18/lib CC=gcc cargo build --features sinks-tsurugidb
  ```

#### 2. 接続エラー: "Failed to connect to Tsurugi" または "Wire::pull() slot=0 timeout"

**問題**: TsurugiDBサーバーへの接続に失敗する、またはタイムアウトが発生する

**原因**:
- TsurugiDBサーバーが起動していない
- エンドポイントが間違っている
- ネットワークの問題（ファイアウォール、ポートが閉じているなど）
- タイムアウト設定が短すぎる

**解決方法**:

1. **TsurugiDBサーバーの起動確認**
   ```bash
   # TsurugiDBサーバーのプロセスを確認
   ps aux | grep tsurugi
   
   # ポートがリッスンしているか確認
   netstat -tlnp | grep 12345
   # または
   ss -tlnp | grep 12345
   ```

2. **エンドポイントの確認**
   ```bash
   # 環境変数を確認
   echo $TSURUGI_ENDPOINT
   
   # デフォルトは tcp://localhost:12345
   # 別のホスト/ポートを使用する場合
   export TSURUGI_ENDPOINT="tcp://your-host:port"
   ```

3. **接続テスト**
   ```bash
   # TsurugiDBのクライアントツールで接続をテスト
   # （TsurugiDBのクライアントツールが利用可能な場合）
   
   # または、telnetでポートが開いているか確認
   telnet localhost 12345
   ```

4. **タイムアウト設定の確認**
   - デフォルトのタイムアウトは10秒です
   - サーバーが遅い場合やネットワークが遅い場合は、タイムアウトが発生する可能性があります
   - TsurugiDBサーバーのログを確認して、リクエストが到達しているか確認してください

5. **ネットワーク設定の確認**
   ```bash
   # ファイアウォール設定を確認
   sudo ufw status
   # または
   sudo iptables -L
   
   # 必要に応じてポートを開放
   sudo ufw allow 12345/tcp
   ```

6. **TsurugiDBサーバーのログ確認**
   - TsurugiDBサーバーのログを確認して、接続試行が記録されているか確認
   - エラーメッセージがないか確認

#### 3. テーブル作成エラー: "Failed to create table"

**問題**: テスト用テーブルの作成に失敗する

**解決方法**:
- データベースへの接続権限を確認
- データベースが存在することを確認
- TsurugiDBサーバーのログを確認
- 既存のテーブルとの名前の競合を確認（ランダム名を使用しているため、通常は発生しません）

#### 4. テストがスキップされる

**問題**: テストが実行されない

**解決方法**:
- `--ignored` フラグを指定しているか確認（`#[ignore]` が付いているテストの場合）
- `--features sinks-tsurugidb,tsurugidb_sink-integration-tests` を指定しているか確認（両方のフィーチャーが必要）
- `--lib sinks::tsurugidb::integration_tests` を使用しているか確認（`--test integration` ではなく）
- テスト名が正しいか確認

```bash
# すべてのテスト（ignoredを含む）をリスト表示
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --list --ignored

# すべてのテストをリスト表示（ignoredを含む）
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --list
```

#### 5. タイムアウトエラー

**問題**: テストがタイムアウトする

**解決方法**:
- TsurugiDBサーバーのパフォーマンスを確認
- ネットワークの遅延を確認
- テストのタイムアウト設定を確認（デフォルト: 10秒）

#### 6. バッチステータスの確認エラー

**問題**: `BatchStatus::Delivered` が期待されない

**解決方法**:
- TsurugiDBサーバーへのデータ挿入が成功しているか確認
- トランザクションのコミットが正常に完了しているか確認
- ログを確認してエラーメッセージを確認

### デバッグ方法

#### 詳細なログを有効にする

```bash
RUST_BACKTRACE=full \
VECTOR_LOG=vector=debug,tsurugidb=debug \
TSURUGI_ENDPOINT="tcp://localhost:12345" \
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --ignored --nocapture
```

#### 単一のテストを実行してデバッグ

```bash
# 特定のテストのみを実行
RUST_BACKTRACE=full \
VECTOR_LOG=vector=debug \
  cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests::insert_single_event -- --ignored --nocapture
```

#### TsurugiDBサーバーのログを確認

TsurugiDBサーバーのログを確認して、接続やクエリの実行状況を確認します。

---

## 参考情報

### 関連ファイル

- **テスト実装**: `src/sinks/tsurugidb/integration_tests.rs`
- **設定**: `src/sinks/tsurugidb/config.rs`
- **Sink実装**: `src/sinks/tsurugidb/sink.rs`
- **サービス実装**: `src/sinks/tsurugidb/service.rs`
- **テストユーティリティ**: `src/test_util/integration.rs` (tsurugiモジュール)

### Cargo.toml の設定

結合テストを実行するには、以下の2つのフィーチャーが必要です：

```toml
# Cargo.toml の [features] セクション
tsurugidb_sink-integration-tests = ["sinks-tsurugidb"]
```

**重要**: テストを実行する際は、`sinks-tsurugidb` と `tsurugidb_sink-integration-tests` の両方を有効にする必要があります：

```bash
cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests
```

### 環境変数の詳細

#### TSURUGI_ENDPOINT

- **型**: 文字列
- **デフォルト**: `tcp://localhost:12345`
- **形式**: `tcp://host:port`
- **説明**: TsurugiDBサーバーへの接続エンドポイント

### テストの実装詳細

テストは以下の手順で実行されます：

1. **設定の準備**: `prepare_config()` 関数でランダムなテーブル名とエンドポイントを使用して設定を準備
2. **テーブルの作成**: `create_test_table()` 関数でテスト用のテーブルを作成
3. **Sinkの構築**: 設定からSinkを構築
4. **イベントの送信**: テストイベントをSinkに送信
5. **結果の検証**: バッチステータスやエラーを検証

### 関連ドキュメント

- **Vector開発者向けドキュメント**: `docs/DEVELOPING-ja.md`
- **TsurugiDB Sink設計ドキュメント**: `src/sinks/tsurugidb/doc/design.md`
- **作業ログ**: `src/sinks/tsurugidb/doc/work_log.md`

---

## まとめ

この手順書に従うことで、TsurugiDB Sinkの結合テストを正常に実行できます。

### クイックスタート

1. **TsurugiDBサーバーを起動**
   ```bash
   # TsurugiDBサーバーの起動方法は、TsurugiDBのドキュメントを参照してください
   # デフォルトでは localhost:12345 でリッスンしている必要があります
   ```

2. **サーバーの起動確認**
   ```bash
   # ポートがリッスンしているか確認
   netstat -tlnp | grep 12345
   # または
   ss -tlnp | grep 12345
   ```

3. **環境変数を設定（必要に応じて）**
   ```bash
   export TSURUGI_ENDPOINT="tcp://localhost:12345"
   ```

4. **テストを実行**
   ```bash
   cargo test --features sinks-tsurugidb,tsurugidb_sink-integration-tests --lib sinks::tsurugidb::integration_tests -- --ignored
   ```

**注意**: テストが `Wire::pull() slot=0 timeout` エラーで失敗する場合、TsurugiDBサーバーが起動していないか、接続できない状態です。上記の手順1-2を確認してください。

### サポート

問題が発生した場合：

1. このドキュメントのトラブルシューティングセクションを確認
2. テストのログを確認
3. TsurugiDBサーバーのログを確認
4. プロジェクトのIssueを検索
5. 必要に応じて新しいIssueを作成

---

**最終更新**: 2025年12月15日
