# Tsurugi用シンクコンポーネント

## 要件

- [Tsurugi用Rustクライアント](https://github.com/project-tsurugi/tsubakuro-rust) を用いてシンクコンポーネントを開発する
- TsurugiはRDBMSなのでpostgres用シンクに準じる機能を実装する

## 作業計画

### 1. プロジェクト構造の確認と基本ファイルの作成

postgresシンクの構造を参考に、以下のファイルを作成します：

- `src/sinks/tsurugidb/mod.rs` - モジュールのエクスポート
- `src/sinks/tsurugidb/config.rs` - 設定構造体とSinkConfigトレイトの実装
- `src/sinks/tsurugidb/service.rs` - Serviceトレイトの実装、RetryLogic、Request/Response型
- `src/sinks/tsurugidb/sink.rs` - StreamSinkトレイトの実装

### 2. Cargo.tomlへの依存関係追加

- `tsubakuro-rust-core` の依存関係を追加
- `sinks-tsurugidb`フィーチャーを追加（`sinks-postgres`と同様のパターン）

### 3. config.rsの実装

postgresシンクの`PostgresConfig`を参考に、以下を実装：

- `TsurugiConfig`構造体
  - `endpoint`: Tsurugiへの接続文字列
  - `table`: データを挿入するテーブル名
  - `pool_size`: 接続プールサイズ（オプション、デフォルト値設定）
  - `transaction_type`: トランザクションタイプ
  - `batch`: イベントバッチ設定
  - `request`: TowerRequestConfig
  - `acknowledgements`: AcknowledgementsConfig
- `GenerateConfig`トレイトの実装
- `SinkConfig`トレイトの実装
  - `build()`: Tsurugiクライアントの初期化、サービスとシンクの構築
  - `input()`: 入力タイプの定義
  - `acknowledgements()`: 確認応答設定の返却
- `healthcheck()`関数: 接続確認用

### 4. service.rsの実装

postgresシンクの`PostgresService`を参考に、以下を実装：

- `TsurugiRetryLogic`: リトライロジックの実装
  - `is_retriable_error()`: リトライ可能なエラーの判定
- `TsurugiService`: メインのサービス実装
  - Tsurugiクライアントの保持
  - テーブル名とエンドポイントの保持
- `TsurugiRequest`: リクエスト型
  - `TryFrom<Vec<Event>>`の実装
  - `Finalizable`トレイトの実装
  - `MetaDescriptive`トレイトの実装
- `TsurugiResponse`: レスポンス型
  - `DriverResponse`トレイトの実装
- `TsurugiServiceError`: エラー型
  - `Snafu`を使用したエラー定義
- `Service<TsurugiRequest>`トレイトの実装
  - `call()`: イベントのシリアライズとTsurugiへの挿入処理
  - `jsonb_populate_recordset`相当の機能をTsurugiクライアントで実装

### 5. sink.rsの実装

postgresシンクの`PostgresSink`を参考に、以下を実装：

- `TsurugiSink`構造体
  - `service`: `Svc<TsurugiService, TsurugiRetryLogic>`
  - `batch_settings`: `BatcherSettings`
- `StreamSink<Event>`トレイトの実装
  - `run()`: イベントストリームの処理
  - バッチ処理とリクエスト変換の実装

### 6. mod.rsの実装

- モジュールの宣言とエクスポート
- `TsurugiConfig`の公開

### 7. sinks/mod.rsへの登録

- `#[cfg(feature = "sinks-tsurugidb")]`条件付きでモジュールを追加
- `pub mod tsurugidb;`の追加

### 8. テストの実装

- `config.rs`内のユニットテスト
  - `generate_config()`テスト
  - `parse_config()`テスト
- 統合テスト（必要に応じて）

### 実装上の注意点

1. **Tsurugiクライアントの使用方法**
   - tsubakuro-rustのAPIドキュメントとサンプルコードを参照
   - 接続プールの管理方法を確認
   - SQL実行方法を確認（postgresの`jsonb_populate_recordset`相当の機能）

2. **postgresシンクとの違い**
   - sqlxの代わりにtsubakuro-rustを使用
   - 接続文字列の形式が異なる可能性
   - SQL構文が異なる可能性（Tsurugi固有の構文）

3. **エラーハンドリング**
   - Tsurugi固有のエラータイプに対応
   - リトライ可能なエラーとそうでないエラーの適切な分類

4. **パフォーマンス**
   - バッチ処理の最適化
   - 接続プールの適切なサイズ設定

