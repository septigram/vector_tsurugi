# Tsurugi用シンクコンポーネント 設計書

## 概要

この設計書は、VectorのTsurugi用シンクコンポーネントの実装設計を記述します。
postgresシンクを参考にし、tsubakuro-rust-coreを使用してTsurugiデータベースへのデータ挿入機能を実装します。

## アーキテクチャ

### 全体構成

```
┌─────────────┐
│   Event     │
│   Stream    │
└──────┬──────┘
       │
       ▼
┌─────────────────┐
│  TsurugiSink    │  ← StreamSinkトレイト実装
│  (バッチ処理)    │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│ TsurugiService  │  ← Serviceトレイト実装
│  (Tower層)      │
└──────┬──────────┘
       │
       ▼
┌─────────────────┐
│  tsubakuro-     │
│  rust-core      │
│  (Session/      │
│   SqlClient)    │
└─────────────────┘
```

### コンポーネント構成

1. **TsurugiSink** (`sink.rs`)
   - `StreamSink<Event>`トレイトの実装
   - イベントストリームのバッチ処理
   - `TsurugiRequest`への変換

2. **TsurugiService** (`service.rs`)
   - `Service<TsurugiRequest>`トレイトの実装
   - Tsurugiへの接続管理
   - SQL実行とトランザクション管理

3. **TsurugiConfig** (`config.rs`)
   - `SinkConfig`トレイトの実装
   - 設定のパースと検証
   - サービスとシンクの構築

4. **TsurugiRetryLogic** (`service.rs`)
   - `RetryLogic`トレイトの実装
   - リトライ可能なエラーの判定

## 詳細設計

### 1. config.rs

#### TsurugiConfig構造体

```rust
#[configurable_component(sink("tsurugidb", "Deliver log data to a Tsurugi database."))]
#[derive(Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct TsurugiConfig {
    /// Tsurugiサーバーへの接続エンドポイントURL
    /// 形式: "tcp://host:port"
    pub endpoint: String,

    /// データを挿入するテーブル名
    /// 注意: SQLインジェクション攻撃に対して脆弱です。
    /// Vectorは検証やサニタイズを行わないため、信頼できない入力は使用しないでください。
    pub table: String,

    /// 接続プールサイズ（将来の拡張用、現在は未使用）
    /// デフォルト: 5
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// トランザクションタイプ
    /// - "occ": Optimistic Concurrency Control（楽観的並行制御）
    /// - "ltx": Long Transaction（長トランザクション）
    /// デフォルト: "occ"
    #[serde(default = "default_transaction_type")]
    pub transaction_type: String,

    /// イベントバッチ設定
    #[configurable(derived)]
    #[serde(default)]
    pub batch: BatchConfig<RealtimeSizeBasedDefaultBatchSettings>,

    /// Towerリクエスト設定
    #[configurable(derived)]
    #[serde(default)]
    pub request: TowerRequestConfig,

    /// 確認応答設定
    #[configurable(derived)]
    #[serde(
        default,
        deserialize_with = "crate::serde::bool_or_struct",
        skip_serializing_if = "crate::serde::is_default"
    )]
    pub acknowledgements: AcknowledgementsConfig,
}
```

#### 設定項目の説明

- **endpoint**: Tsurugiサーバーへの接続エンドポイント
  - 形式: `tcp://host:port` (例: `tcp://localhost:12345`)
  - `ConnectionOption::set_endpoint_url()`で使用

- **table**: データ挿入先のテーブル名
  - SQLインジェクション対策が必要（信頼できない入力は使用不可）
  - テーブル名はパラメータ化できないため、直接SQLに埋め込む

- **pool_size**: 接続プールサイズ（将来の拡張用）
  - 現在のtsubakuro-rust-coreは接続プールを提供していないため、現時点では未使用
  - 将来的に接続プールが実装された場合に備えて設定項目として保持

- **transaction_type**: トランザクションタイプ
  - `"occ"`: Optimistic Concurrency Control（楽観的並行制御）
    - 短いトランザクションに適している
    - デフォルト値
  - `"ltx"`: Long Transaction（長トランザクション）
    - 長いトランザクションや書き込み保存テーブルを指定する場合に使用
  - 内部的には`TransactionType::Short`（OCC）または`TransactionType::Long`（LTX）に変換される

#### ヘルパー関数

```rust
fn default_transaction_type() -> String {
    "occ".to_string()
}

fn parse_transaction_type(s: &str) -> Result<TransactionType, String> {
    match s.to_lowercase().as_str() {
        "occ" => Ok(TransactionType::Short),
        "ltx" => Ok(TransactionType::Long),
        _ => Err(format!("Invalid transaction type: {}. Must be 'occ' or 'ltx'", s)),
    }
}
```

#### SinkConfigトレイトの実装

```rust
#[async_trait::async_trait]
#[typetag::serde(name = "tsurugidb")]
impl SinkConfig for TsurugiConfig {
    async fn build(&self, _cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        // 1. ConnectionOptionの作成
        let mut connection_option = ConnectionOption::new();
        connection_option.set_endpoint_url(&self.endpoint)?;
        connection_option.set_application_name("Vector Tsurugi Sink");
        connection_option.set_default_timeout(Duration::from_secs(10));

        // 2. セッションの作成（接続プールの代わり）
        // 注意: 現在のtsubakuro-rust-coreは接続プールを提供していないため、
        // セッションを直接保持する
        let session = Session::connect(&connection_option).await?;

        // 3. ヘルスチェックの作成
        let healthcheck = healthcheck(session.clone()).boxed();

        // 4. バッチ設定とリクエスト設定
        let batch_settings = self.batch.into_batcher_settings()?;
        let request_settings = self.request.into_settings();

        // 5. トランザクションタイプのパース
        let transaction_type = parse_transaction_type(&self.transaction_type)
            .map_err(|e| crate::Error::from(e))?;

        // 6. サービスの作成
        let service = TsurugiService::new(
            session,
            self.table.clone(),
            self.endpoint.clone(),
            transaction_type,
        );

        // 7. Tower ServiceBuilderでラップ
        let service = ServiceBuilder::new()
            .settings(request_settings, TsurugiRetryLogic)
            .service(service);

        // 8. シンクの作成
        let sink = TsurugiSink::new(service, batch_settings);

        Ok((VectorSink::from_event_streamsink(sink), healthcheck))
    }

    fn input(&self) -> Input {
        Input::all()
    }

    fn acknowledgements(&self) -> &AcknowledgementsConfig {
        &self.acknowledgements
    }
}
```

#### ヘルスチェック実装

```rust
async fn healthcheck(session: Arc<Session>) -> crate::Result<()> {
    let client: SqlClient = session.make_client();
    
    // 簡単なSELECT文で接続確認
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::ReadOnly);
    
    let transaction = client.start_transaction(&transaction_option).await?;
    
    // SELECT 1 を実行して接続を確認
    let sql = "SELECT 1";
    let mut query_result = client.query(&transaction, sql).await?;
    
    // 結果を確認（1行取得できればOK）
    if query_result.next_row().await? {
        // 正常
    }
    
    query_result.close().await?;
    transaction.close().await?;
    
    Ok(())
}
```

### 2. service.rs

#### TsurugiService構造体

```rust
#[derive(Clone)]
pub struct TsurugiService {
    session: Arc<Session>,
    table: String,
    endpoint: String,
    transaction_type: TransactionType,
}

impl TsurugiService {
    pub const fn new(
        session: Arc<Session>,
        table: String,
        endpoint: String,
        transaction_type: TransactionType,
    ) -> Self {
        Self {
            session,
            table,
            endpoint,
            transaction_type,
        }
    }
}
```

**設計上の考慮事項**:
- `Session`は`Arc`で包まれているため、`Clone`が可能
- 接続プールがないため、セッションを直接保持
- 将来的に接続プールが実装された場合は、プールへの参照に変更
- トランザクションタイプは設定から取得し、サービスに保持

#### Serviceトレイトの実装

```rust
impl Service<TsurugiRequest> for TsurugiService {
    type Response = TsurugiResponse;
    type Error = TsurugiServiceError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: TsurugiRequest) -> Self::Future {
        let service = self.clone();
        let future = async move {
            // 1. SqlClientの取得
            let client: SqlClient = service.session.make_client();
            
            // 2. トランザクションの開始
            let mut transaction_option = TransactionOption::new();
            transaction_option.set_transaction_type(service.transaction_type);
            let transaction = client.start_transaction(&transaction_option).await
                .context(TsurugiSnafu)?;
            
            // 3. イベントのシリアライズ
            let json_serializer = JsonSerializerConfig::default().build();
            let serialized_values: Vec<serde_json::Value> = request
                .events
                .into_iter()
                .map(|event| json_serializer.to_json_value(event))
                .collect::<Result<Vec<_>, _>>()
                .context(VectorCommonSnafu)?;
            
            // 4. 個別INSERT文の生成と実行
            // 注意: TsurugiはPostgreSQLのjsonb_populate_recordset相当の機能を
            // サポートしていないため、各イベントを個別のINSERT文として実行する
            for value in serialized_values {
                // JSONオブジェクトからINSERT文を構築
                // 例: {"col1": "val1", "col2": 123} -> INSERT INTO table (col1, col2) VALUES ('val1', 123)
                let sql = build_insert_sql(&service.table, &value)
                    .context(JsonSerializationSnafu)?;
                
                client.execute(&transaction, &sql).await
                    .context(TsurugiSnafu)?;
            }
            
            // 6. コミット
            let commit_option = CommitOption::default();
            client.commit(&transaction, &commit_option).await
                .context(TsurugiSnafu)?;
            
            // 7. トランザクションのクローズ
            transaction.close().await
                .context(TsurugiSnafu)?;
            
            // 8. メトリクスの発行
            emit!(EndpointBytesSent {
                byte_size: request.metadata.request_encoded_size(),
                protocol: TSURUGI_PROTOCOL,
                endpoint: &service.endpoint,
            });
            
            Ok(TsurugiResponse {
                metadata: request.metadata,
            })
        };
        
        Box::pin(future)
    }
}
```

**SQL挿入方法**:

TsurugiはPostgreSQLの`jsonb_populate_recordset`相当の機能をサポートしていないため、
各イベントを個別のINSERT文として実行します。

```rust
// 各イベントを個別のINSERT文に変換して実行
for value in serialized_values {
    let sql = build_insert_sql(&service.table, &value)?;
    client.execute(&transaction, &sql).await?;
}
```

`build_insert_sql`関数は、JSONオブジェクトからINSERT文を生成します：
- JSONオブジェクトのキーをカラム名として使用
- 値は適切にエスケープしてSQLリテラルに変換
- 例: `{"col1": "val1", "col2": 123}` → `INSERT INTO table (col1, col2) VALUES ('val1', 123)`

**パフォーマンス考慮事項**:
- バッチ内のすべてのイベントを1つのトランザクションで実行することで、パフォーマンスを最適化
- 大量のイベントを処理する場合は、バッチサイズの調整を検討

#### TsurugiRetryLogic

```rust
#[derive(Clone)]
pub struct TsurugiRetryLogic;

impl RetryLogic for TsurugiRetryLogic {
    type Error = TsurugiServiceError;
    type Request = TsurugiRequest;
    type Response = TsurugiResponse;

    fn is_retriable_error(&self, error: &Self::Error) -> bool {
        match error {
            TsurugiServiceError::Tsurugi { source } => {
                match source {
                    // I/Oエラーはリトライ可能
                    TgError::IoError(_, _) => true,
                    // タイムアウトエラーはリトライ可能
                    TgError::TimeoutError(_) => true,
                    // クライアントエラーの一部はリトライ可能
                    TgError::ClientError(msg, _) => {
                        // 接続関連のエラーはリトライ可能
                        msg.contains("connection") || msg.contains("network")
                    }
                    // サーバーエラーは診断コードで判定
                    TgError::ServerError(_, _, code, _) => {
                        // 一時的なエラー（例: デッドロック、タイムアウト）はリトライ可能
                        // 診断コードの詳細はTsurugiの仕様に依存
                        // 現時点では、一般的な一時的エラーを想定
                        true // 暫定: 詳細な判定は実装時に追加
                    }
                }
            }
            TsurugiServiceError::VectorCommon { .. } => false,
            TsurugiServiceError::JsonSerialization { .. } => false,
        }
    }
}
```

#### TsurugiRequest

```rust
#[derive(Clone)]
pub struct TsurugiRequest {
    pub events: Vec<Event>,
    pub finalizers: EventFinalizers,
    pub metadata: RequestMetadata,
}

impl TryFrom<Vec<Event>> for TsurugiRequest {
    type Error = String;

    fn try_from(mut events: Vec<Event>) -> Result<Self, Self::Error> {
        let finalizers = events.take_finalizers();
        let metadata_builder = RequestMetadataBuilder::from_events(&events);
        let events_size = NonZeroUsize::new(events.estimated_json_encoded_size_of().get())
            .ok_or("payload should never be zero length")?;
        let metadata = metadata_builder.with_request_size(events_size);
        Ok(TsurugiRequest {
            events,
            finalizers,
            metadata,
        })
    }
}

impl Finalizable for TsurugiRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        self.finalizers.take_finalizers()
    }
}

impl MetaDescriptive for TsurugiRequest {
    fn get_metadata(&self) -> &RequestMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut RequestMetadata {
        &mut self.metadata
    }
}
```

#### TsurugiResponse

```rust
pub struct TsurugiResponse {
    metadata: RequestMetadata,
}

impl DriverResponse for TsurugiResponse {
    fn event_status(&self) -> EventStatus {
        EventStatus::Delivered
    }

    fn events_sent(&self) -> &GroupedCountByteSize {
        self.metadata.events_estimated_json_encoded_byte_size()
    }

    fn bytes_sent(&self) -> Option<usize> {
        Some(self.metadata.request_encoded_size())
    }
}
```

#### TsurugiServiceError

```rust
#[derive(Debug, Snafu)]
pub enum TsurugiServiceError {
    #[snafu(display("Tsurugi database error: {source}"))]
    Tsurugi { source: TgError },

    #[snafu(display("Serialization error: {source}"))]
    VectorCommon { source: vector_common::Error },

    #[snafu(display("JSON serialization error: {source}"))]
    JsonSerialization { source: serde_json::Error },
}
```

### 3. sink.rs

```rust
use super::service::{TsurugiRequest, TsurugiRetryLogic, TsurugiService};
use crate::sinks::prelude::*;

pub struct TsurugiSink {
    service: Svc<TsurugiService, TsurugiRetryLogic>,
    batch_settings: BatcherSettings,
}

impl TsurugiSink {
    pub const fn new(
        service: Svc<TsurugiService, TsurugiRetryLogic>,
        batch_settings: BatcherSettings,
    ) -> Self {
        Self {
            service,
            batch_settings,
        }
    }

    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        input
            .batched(self.batch_settings.as_byte_size_config())
            .filter_map(|events| async move {
                match TsurugiRequest::try_from(events) {
                    Ok(request) => Some(request),
                    Err(e) => {
                        warn!(
                            message = "Error creating tsurugi sink's request.",
                            error = %e
                        );
                        None
                    }
                }
            })
            .into_driver(self.service)
            .run()
            .await
    }
}

#[async_trait::async_trait]
impl StreamSink<Event> for TsurugiSink {
    async fn run(mut self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
```

### 4. mod.rs

```rust
mod config;
#[cfg(all(test, feature = "tsurugidb_sink-integration-tests"))]
mod integration_tests;
mod service;
mod sink;

pub use self::config::TsurugiConfig;
```

## エラーハンドリング

### エラー分類

1. **リトライ可能なエラー**:
   - I/Oエラー（ネットワーク障害など）
   - タイムアウトエラー
   - 接続関連のクライアントエラー
   - 一時的なサーバーエラー（デッドロックなど）

2. **リトライ不可能なエラー**:
   - シリアライズエラー
   - SQL構文エラー
   - 制約違反エラー
   - 認証エラー

### エラー処理の流れ

```
エラー発生
    │
    ▼
TsurugiServiceErrorに変換
    │
    ▼
TsurugiRetryLogicで判定
    │
    ├─ リトライ可能 → Tower層でリトライ
    │
    └─ リトライ不可能 → エラーとして返却
```

## パフォーマンス考慮事項

### バッチ処理

- postgresシンクと同様に、イベントをバッチにまとめて挿入
- バッチサイズは`BatchConfig`で制御
- デフォルト設定は`RealtimeSizeBasedDefaultBatchSettings`を使用

### トランザクション管理

- 各バッチごとに1つのトランザクションを使用
- トランザクションタイプは`TransactionType::Short`（OCC）を使用
- バッチ内のすべてのイベントが成功した場合のみコミット

### 接続管理

- 現在のtsubakuro-rust-coreは接続プールを提供していない
- セッションを直接保持し、すべてのリクエストで同じセッションを使用
- 将来的に接続プールが実装された場合は移行を検討

## セキュリティ考慮事項

### SQLインジェクション対策

- テーブル名は設定ファイルから取得し、信頼できるソースからのみ使用
- イベントデータはJSONとしてシリアライズし、SQLパラメータとして渡す
- テーブル名はパラメータ化できないため、設定値の検証が重要

### 認証

- 現在の実装では認証情報の設定は未実装
- 将来的に`Credential`を使用した認証を追加可能
- 接続文字列に認証情報を含める方法も検討可能

## テスト戦略

### ユニットテスト

1. **config.rs**:
   - `generate_config()`テスト
   - `parse_config()`テスト
   - 不正な設定値の検証

2. **service.rs**:
   - エラー変換のテスト
   - リトライロジックのテスト

### 統合テスト

1. **基本的な挿入テスト**:
   - 単一イベントの挿入
   - バッチイベントの挿入

2. **エラーハンドリングテスト**:
   - 接続エラー時の動作
   - タイムアウト時の動作
   - 制約違反時の動作

3. **パフォーマンステスト**:
   - 大量データの挿入
   - バッチサイズの最適化

## 将来の拡張

### 接続プール対応

tsubakuro-rust-coreに接続プールが実装された場合：

```rust
pub struct TsurugiService {
    connection_pool: ConnectionPool, // 新しい型
    table: String,
    endpoint: String,
}
```

### 認証対応

```rust
pub struct TsurugiConfig {
    // ...
    pub credential: Option<CredentialConfig>,
}
```

## 依存関係

### Cargo.tomlへの追加

```toml
[dependencies]
tsubakuro-rust-core = { version = "0.x", default-features = false, features = ["with_chrono"] }

[features]
sinks-tsurugidb = ["dep:tsubakuro-rust-core"]
```

### 機能フラグ

- `with_chrono`: 日時型のサポート（推奨）
- `with_bigdecimal` / `with_rust_decimal`: 高精度数値型のサポート（必要に応じて）

## 実装の優先順位

1. **Phase 1: 基本実装**
   - config.rs, service.rs, sink.rs, mod.rsの基本実装
   - 個別INSERT文によるデータ挿入
   - トランザクションタイプ（OCC/LTX）の設定対応
   - 基本的なエラーハンドリング

2. **Phase 2: 最適化**
   - リトライロジックの改善
   - パフォーマンスチューニング
   - バッチサイズの最適化

3. **Phase 3: 拡張機能**
   - 認証サポート
   - 接続プール対応（実装された場合）

## 参考資料

- [plan.md](./plan.md) - 作業計画
- [tsubakuro-rust-core仕様書](./tsubakuro-rust-core/specs.md) - tsubakuro-rust-coreの仕様
- postgresシンク実装 (`src/sinks/postgres/`) - 参考実装
