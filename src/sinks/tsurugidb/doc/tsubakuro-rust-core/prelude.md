# prelude モジュール仕様

## 概要

`prelude` モジュールは、tsubakuro-rust-core の主要な型とトレイトをエクスポートするモジュールです。このモジュールを使用することで、ライブラリの主要な機能に簡単にアクセスできます。

## 使用方法

```rust
use tsubakuro_rust_core::prelude::*;
```

このように使用することで、以下の主要な型とトレイトがインポートされます。

## エクスポートされる型

### エラー関連

#### TgError

エラー型。詳細は [error.md](./error.md) を参照してください。

```rust
pub use crate::error::*;
```

#### DiagnosticCode

診断コード型。詳細は [error.md](./error.md) を参照してください。

### ジョブ関連

#### Job<T>

非同期ジョブ型。詳細は [job.md](./job.md) を参照してください。

```rust
pub use crate::job::Job;
```

#### CancelJob

キャンセルジョブ型。詳細は [job.md](./job.md) を参照してください。

```rust
pub use crate::job::cancel_job::CancelJob;
```

### セッション関連

#### Session

セッション型。詳細は [session.md](./session.md) を参照してください。

```rust
pub use crate::session::Session;
```

#### ConnectionOption

接続オプション型。詳細は [session.md](./session.md) を参照してください。

```rust
pub use crate::session::option::*;
```

#### Credential

認証情報型。詳細は [session.md](./session.md) を参照してください。

```rust
pub use crate::session::credential::*;
```

#### Endpoint

エンドポイント型。詳細は [session.md](./session.md) を参照してください。

```rust
pub use crate::session::endpoint::*;
```

### トランザクション関連

#### Transaction

トランザクション型。詳細は [transaction.md](./transaction.md) を参照してください。

```rust
pub use crate::transaction::Transaction;
```

#### TransactionOption

トランザクションオプション型。詳細は [transaction.md](./transaction.md) を参照してください。

```rust
pub use crate::transaction::option::*;
```

#### CommitOption

コミットオプション型。詳細は [transaction.md](./transaction.md) を参照してください。

```rust
pub use crate::jogasaki::proto::sql::request::CommitOption;
```

#### TransactionType

トランザクションタイプ列挙型。

```rust
pub use crate::jogasaki::proto::sql::request::TransactionType;
```

#### CommitType

コミットタイプ列挙型。

```rust
pub use crate::jogasaki::proto::sql::request::CommitStatus as CommitType;
```

#### TransactionStatus

トランザクションステータス列挙型。

```rust
pub use crate::jogasaki::proto::sql::response::TransactionStatus;
```

#### TransactionStatusWithMessage

トランザクションステータスとメッセージを保持する型。詳細は [transaction.md](./transaction.md) を参照してください。

```rust
pub use crate::transaction::status::*;
```

#### TransactionErrorInfo

トランザクションエラー情報型。詳細は [transaction.md](./transaction.md) を参照してください。

```rust
pub use crate::transaction::error_info::*;
```

### SQLサービス関連

#### SqlClient

SQLサービスクライアント型。詳細は [service.md](./service.md) を参照してください。

```rust
pub use crate::service::sql::*;
```

#### SqlPreparedStatement

プリペアドステートメント型。詳細は [service.md](./service.md) を参照してください。

#### SqlExecuteResult

SQL実行結果型。詳細は [service.md](./service.md) を参照してください。

#### SqlQueryResult

SQLクエリ結果型。詳細は [service.md](./service.md) を参照してください。

#### SqlParameter

SQLパラメータ型。詳細は [service.md](./service.md) を参照してください。

```rust
pub use crate::jogasaki::proto::sql::request::Parameter as SqlParameter;
```

#### SqlPlaceholder

SQLプレースホルダ型。詳細は [service.md](./service.md) を参照してください。

```rust
pub use crate::jogasaki::proto::sql::request::Placeholder as SqlPlaceholder;
```

#### SqlExplainResult

SQL実行計画結果型。詳細は [service.md](./service.md) を参照してください。

#### TableList

テーブル一覧型。詳細は [service.md](./service.md) を参照してください。

#### TableMetadata

テーブルメタデータ型。詳細は [service.md](./service.md) を参照してください。

### データ型関連

#### AtomType

原子型列挙型。

```rust
pub use crate::jogasaki::proto::sql::common::AtomType;
```

#### SqlColumn

SQLカラム型。

```rust
pub use crate::jogasaki::proto::sql::common::Column as SqlColumn;
```

#### SqlQueryResultMetadata

SQLクエリ結果メタデータ型。

```rust
pub use crate::jogasaki::proto::sql::response::ResultSetMetadata as SqlQueryResultMetadata;
```

#### SqlCounterType

SQLカウンタータイプ列挙型。

```rust
pub use crate::jogasaki::proto::sql::response::execute_result::CounterType as SqlCounterType;
```

### 日時・数値型関連

以下の型は、対応する機能フラグが有効な場合に使用できます：

- `TgDate` - 日付型
- `TgTimeOfDay` - 時刻型
- `TgTimeOfDayWithTimeZone` - タイムゾーン付き時刻型
- `TgTimePoint` - タイムポイント型
- `TgTimePointWithTimeZone` - タイムゾーン付きタイムポイント型
- `TgDecimal` - 10進数型
- `TgDecimalI128` - i128ベースの10進数型
- `TgBlob` / `TgBlobReference` - BLOB型
- `TgClob` / `TgClobReference` - CLOB型

詳細は [service.md](./service.md) を参照してください。

### その他の型

#### ServiceClient

サービスクライアントトレイト。詳細は [service.md](./service.md) を参照してください。

```rust
pub use crate::service::*;
```

#### ServiceMessageVersion

サービスメッセージバージョントレイト。詳細は [service.md](./service.md) を参照してください。

#### ShutdownType

シャットダウンタイプ列挙型。

```rust
pub use crate::tateyama::proto::core::request::ShutdownType;
```

#### TransactionPriority

トランザクション優先度列挙型。

```rust
pub use crate::jogasaki::proto::sql::request::TransactionPriority;
```

## 使用例

### 基本的なインポート

```rust
use tsubakuro_rust_core::prelude::*;
```

### セッションの作成

```rust
use std::time::Duration;
use tsubakuro_rust_core::prelude::*;

async fn example() -> Result<(), TgError> {
    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url("tcp://localhost:12345")?;
    connection_option.set_application_name("My App");
    connection_option.set_default_timeout(Duration::from_secs(10));
    
    let session = Session::connect(&connection_option).await?;
    // ...
    Ok(())
}
```

### SQLの実行

```rust
use tsubakuro_rust_core::prelude::*;

async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let sql = "select * from customer";
    let mut query_result = client.query(transaction, sql).await?;
    
    while query_result.next_row().await? {
        // データを処理
    }
    
    query_result.close().await?;
    Ok(())
}
```

## 注意事項

1. **ワイルドカードインポート**: `use tsubakuro_rust_core::prelude::*;` を使用することで、多くの型がスコープにインポートされます。名前の衝突を避けるため、必要に応じて特定の型のみをインポートすることを検討してください。

2. **機能フラグ**: 一部の型（特に日時・数値型）は、対応する機能フラグが有効な場合のみ使用できます。

3. **内部実装**: `prelude` モジュールは、他のモジュールから型を再エクスポートしているだけです。実際の実装は各モジュールにあります。

## 関連ドキュメント

- [error.md](./error.md) - エラー処理
- [job.md](./job.md) - ジョブ管理
- [session.md](./session.md) - セッション管理
- [transaction.md](./transaction.md) - トランザクション管理
- [service.md](./service.md) - SQLサービス
- [util.md](./util.md) - ユーティリティ

