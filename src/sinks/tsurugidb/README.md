# TsurugiDB Sink

TsurugiDB用のVectorシンクコンポーネントです。このシンクは、VectorのイベントストリームをTsurugiデータベースに挿入する機能を提供します。

## 概要

`tsurugidb`シンクは、[tsubakuro-rust-core](https://github.com/project-tsurugi/tsubakuro-rust)を使用してTsurugiデータベースへのデータ挿入機能を実装しています。postgresシンクを参考に設計されており、同様の機能を提供します。

## 機能

- **バッチ処理**: イベントをバッチにまとめて効率的に挿入
- **トランザクション管理**: バッチ単位でトランザクションを管理（OCC/LTX対応）
- **リトライ機能**: 一時的なエラーに対して自動リトライ
- **ヘルスチェック**: 接続状態の確認
- **柔軟な設定**: バッチサイズ、リクエスト設定などのカスタマイズ可能

## 設定

### 基本設定

```toml
[sinks.my_tsurugi_sink]
type = "tsurugidb"
endpoint = "tcp://localhost:12345"
table = "log_events"
```

### 設定項目

| 項目 | 型 | 必須 | デフォルト | 説明 |
|------|-----|------|-----------|------|
| `endpoint` | string | はい | - | Tsurugiサーバーへの接続エンドポイント（形式: `tcp://host:port`） |
| `table` | string | はい | - | データを挿入するテーブル名 |
| `pool_size` | u32 | いいえ | `5` | 接続プールサイズ（将来の拡張用、現在は未使用） |
| `transaction_type` | string | いいえ | `"occ"` | トランザクションタイプ（`"occ"` または `"ltx"`） |
| `batch` | object | いいえ | - | イベントバッチ設定（詳細は[バッチ設定](#バッチ設定)を参照） |
| `request` | object | いいえ | - | Towerリクエスト設定（詳細は[リクエスト設定](#リクエスト設定)を参照） |
| `credential` | object | いいえ | - | 認証情報設定（詳細は[認証設定](#認証設定)を参照） |
| `acknowledgements` | bool/object | いいえ | - | 確認応答設定 |

### トランザクションタイプ

- **`"occ"`** (Optimistic Concurrency Control): 楽観的並行制御。短いトランザクションに適しています（デフォルト）
- **`"ltx"`** (Long Transaction): 長トランザクション。長いトランザクションや書き込み保存テーブルを指定する場合に使用

### バッチ設定

```toml
[sinks.my_tsurugi_sink.batch]
max_bytes = 10485760  # 10MB
max_events = 1000
timeout_secs = 1
```

### リクエスト設定

```toml
[sinks.my_tsurugi_sink.request]
in_flight_limit = 5
rate_limit_duration_secs = 1
rate_limit_num = 10
retry_attempts = 5
retry_max_duration_secs = 300
```

### 認証設定

認証情報の設定はオプションです。以下の3つの認証方式をサポートしています。

#### ユーザー名/パスワード認証

```toml
[sinks.my_tsurugi_sink.credential]
type = "user_password"
user = "admin"
password = "secret123"
```

`password`はオプションです（パスワードなしで接続する場合）。

#### 認証トークン認証

```toml
[sinks.my_tsurugi_sink.credential]
type = "auth_token"
token = "your-auth-token"
```

#### ファイルから認証情報を読み込み

```toml
[sinks.my_tsurugi_sink.credential]
type = "file"
path = "/etc/tsurugi/credentials"
```

ファイル形式はtsubakuro-rust-coreの`Credential::load()`メソッドでサポートされる形式である必要があります。

**注意**: パスワードやトークンは機密情報として扱われ、ログ出力時に自動的にマスクされます。

### 完全な設定例

```toml
[sinks.my_tsurugi_sink]
type = "tsurugidb"
endpoint = "tcp://localhost:12345"
table = "log_events"
transaction_type = "occ"
pool_size = 5

[sinks.my_tsurugi_sink.credential]
type = "user_password"
user = "admin"
password = "secret123"

[sinks.my_tsurugi_sink.batch]
max_bytes = 10485760
max_events = 1000
timeout_secs = 1

[sinks.my_tsurugi_sink.request]
in_flight_limit = 5
rate_limit_duration_secs = 1
rate_limit_num = 10
retry_attempts = 5
retry_max_duration_secs = 300

[sinks.my_tsurugi_sink.acknowledgements]
enabled = true
```

## 使用方法

### 基本的な使用例

```toml
[sources.my_source]
type = "stdin"

[sinks.my_tsurugi_sink]
type = "tsurugidb"
inputs = ["my_source"]
endpoint = "tcp://localhost:12345"
table = "log_events"
```

### イベントデータの形式

イベントはJSON形式でシリアライズされ、JSONオブジェクトのキーがテーブルのカラム名として使用されます。

例: 以下のようなイベント
```json
{
  "message": "Hello, World!",
  "level": "info",
  "timestamp": "2025-12-15T05:34:01.000904Z"
}
```

は、以下のようなINSERT文に変換されます：
```sql
INSERT INTO log_events (message, level, timestamp) VALUES ('Hello, World!', 'info', '2025-12-15 05:34:01')
```

### 特別なフィールド

- **`event_timestamp`**: このフィールドが存在する場合、TIMESTAMP型用の形式（`'YYYY-MM-DD HH:MM:SS'`）に自動変換されます
- **`timestamp`**: 予約語の可能性があるため、`event_timestamp`が存在する場合はスキップされます

## アーキテクチャ

このシンクは以下のコンポーネントで構成されています：

```
Event Stream
    │
    ▼
TsurugiSink (バッチ処理)
    │
    ▼
TsurugiService (Tower層)
    │
    ▼
tsubakuro-rust-core (Session/SqlClient)
    │
    ▼
Tsurugi Database
```

### コンポーネント

1. **TsurugiSink**: `StreamSink<Event>`トレイトの実装。イベントストリームのバッチ処理とリクエスト変換を担当
2. **TsurugiService**: `Service<TsurugiRequest>`トレイトの実装。Tsurugiへの接続管理、SQL実行、トランザクション管理を担当
3. **TsurugiConfig**: `SinkConfig`トレイトの実装。設定のパースと検証、サービスとシンクの構築を担当

## エラーハンドリング

### リトライ可能なエラー

以下のエラーは自動的にリトライされます：

- I/Oエラー（ネットワーク障害など）
- タイムアウトエラー
- 接続関連のエラー
- 一時的なサーバーエラー

### リトライ不可能なエラー

以下のエラーはリトライされません：

- シリアライズエラー
- SQL構文エラー
- 制約違反エラー
- 認証エラー

## セキュリティ考慮事項

### SQLインジェクション対策

- **テーブル名**: 設定ファイルから取得し、信頼できるソースからのみ使用してください。Vectorは検証やサニタイズを行いません
- **イベントデータ**: JSONとしてシリアライズされ、SQLリテラルに適切に変換されます
  - 文字列値はシングルクォートで囲まれ、内部のシングルクォートは`''`にエスケープされます
  - 配列やオブジェクトはJSON文字列としてシリアライズされます

### 認証

認証情報の設定は`credential`フィールドでサポートされています。tsubakuro-rust-core 0.5.0以降で利用可能な`Credential`型を使用して実装されています。

詳細は[認証設定](#認証設定)セクションを参照してください。

## パフォーマンス

### バッチ処理

- イベントはバッチにまとめて挿入されます
- バッチサイズは`BatchConfig`で制御可能です
- デフォルト設定は`RealtimeSizeBasedDefaultBatchSettings`を使用します

### トランザクション管理

- 各バッチごとに1つのトランザクションを使用します
- バッチ内のすべてのイベントが成功した場合のみコミットされます
- トランザクションタイプは設定で選択可能です（OCC/LTX）

### 接続管理

現在のtsubakuro-rust-coreは接続プールを提供していないため、セッションを直接保持し、すべてのリクエストで同じセッションを使用します。将来的に接続プールが実装された場合は移行を検討します。

## 制限事項

1. **接続プール**: 現在のtsubakuro-rust-coreは接続プールを提供していないため、`pool_size`設定は未使用です
2. **バッチ挿入**: TsurugiはPostgreSQLの`jsonb_populate_recordset`相当の機能をサポートしていないため、各イベントを個別のINSERT文として実行します

## テスト

### ユニットテスト

`config.rs`には以下のテストが含まれています：

- `generate_config()`テスト
- `parse_config()`テスト
- カスタム値のパーステスト
- デフォルト値の確認テスト
- 不正な値の検証テスト

### 統合テスト

統合テストは`integration_tests.rs`に実装されています。詳細は[integration_tests.md](./doc/integration_tests.md)を参照してください。

## 参考資料

- [設計書](./doc/design.md) - 詳細な設計仕様
- [作業計画](./doc/plan.md) - 実装計画と作業ログ
- [統合テスト](./doc/integration_tests.md) - 統合テストの詳細
- [tsubakuro-rust-core仕様書](./doc/tsubakuro-rust-core/specs.md) - tsubakuro-rust-coreの仕様

## 将来の拡張

以下の機能を将来追加する予定です：

1. **接続プール対応**: tsubakuro-rust-coreに接続プールが実装された場合の対応
2. **エラー型の詳細化**: `TgError`の直接利用によるより詳細なエラー情報

## 関連リンク

- [Vector公式ドキュメント](https://vector.dev/docs/)
- [Tsurugiプロジェクト](https://github.com/project-tsurugi)
- [tsubakuro-rust-core](https://github.com/project-tsurugi/tsubakuro-rust)
