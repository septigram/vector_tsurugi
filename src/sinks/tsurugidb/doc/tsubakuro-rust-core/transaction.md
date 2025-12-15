# transaction モジュール仕様

## 概要

`transaction` モジュールは、トランザクション管理機能を提供します。

## 主要な型

### Transaction

トランザクションを表す型です。

**注意**: トランザクションを破棄する前に [`Self::close`] を呼び出す必要があります。

#### メソッド

##### transaction_id()

データベースサーバーの存続期間中に一意なトランザクションIDを返します。

```rust
pub fn transaction_id(&self) -> &String
```

##### set_close_timeout()

クローズタイムアウトを設定します。

```rust
pub fn set_close_timeout(&mut self, timeout: Duration)
```

##### close_timeout()

クローズタイムアウトを取得します。

```rust
pub fn close_timeout(&self) -> Duration
```

##### close()

リソースを破棄します。

**注意**: トランザクションを破棄する前に `close` を呼び出す必要があります。

```rust
pub async fn close(&self) -> Result<(), TgError>
```

##### close_for()

タイムアウトを指定してリソースを破棄します。

```rust
pub async fn close_for(&self, timeout: Duration) -> Result<(), TgError>
```

##### is_closed()

このリソースが閉じられているかどうかを確認します。

```rust
pub fn is_closed(&self) -> bool
```

### TransactionOption

トランザクションオプションです。

#### メソッド

##### new()

新しいインスタンスを作成します。

デフォルト値:
- `transaction_type`: `TransactionType::Short` (OCC)
- その他のフィールド: デフォルト値

```rust
pub fn new() -> TransactionOption
```

##### set_transaction_type()

トランザクションタイプを設定します。

```rust
pub fn set_transaction_type(&mut self, transaction_type: TransactionType)
```

##### transaction_type()

トランザクションタイプを取得します。

```rust
pub fn transaction_type(&self) -> TransactionType
```

##### set_transaction_label()

トランザクションラベルを設定します。

```rust
fn set_transaction_label(&mut self, transaction_label: T)
```

`TransactionOptionSetter` トレイト経由で使用します。

##### transaction_label()

トランザクションラベルを取得します。

```rust
pub fn transaction_label(&self) -> Option<&String>
```

##### set_modifies_definitions()

定義変更フラグを設定します（DDL用）。

```rust
pub fn set_modifies_definitions(&mut self, modifies_definitions: bool)
```

##### modifies_definitions()

定義変更フラグを取得します。

```rust
pub fn modifies_definitions(&self) -> bool
```

##### set_write_preserve()

書き込み保存テーブルを設定します（LTX用）。

```rust
fn set_write_preserve(&mut self, table_names: &[T])
```

`TransactionOptionSetter` トレイト経由で使用します。

##### write_preserve()

書き込み保存テーブルを取得します。

```rust
pub fn write_preserve(&self) -> &Vec<String>
```

##### set_inclusive_read_area()

包含読み取り領域を設定します。

```rust
fn set_inclusive_read_area(&mut self, table_names: &[T])
```

`TransactionOptionSetter` トレイト経由で使用します。

##### inclusive_read_area()

包含読み取り領域を取得します。

```rust
pub fn inclusive_read_area(&self) -> &Vec<String>
```

##### set_exclusive_read_area()

排他読み取り領域を設定します。

```rust
fn set_exclusive_read_area(&mut self, table_names: &[T])
```

`TransactionOptionSetter` トレイト経由で使用します。

##### exclusive_read_area()

排他読み取り領域を取得します。

```rust
pub fn exclusive_read_area(&self) -> &Vec<String>
```

##### set_scan_parallel()

スキャン並列度を設定します。

```rust
pub fn set_scan_parallel(&mut self, scan_parallel: i32)
```

**since**: 0.2.0

##### scan_parallel()

スキャン並列度を取得します。

```rust
pub fn scan_parallel(&self) -> Option<i32>
```

**since**: 0.2.0

##### set_priority()

優先度を設定します。

```rust
pub fn set_priority(&mut self, priority: TransactionPriority)
```

##### priority()

優先度を取得します。

```rust
pub fn priority(&self) -> TransactionPriority
```

##### set_close_timeout()

クローズタイムアウトを設定します。

```rust
pub fn set_close_timeout(&mut self, timeout: Duration)
```

##### close_timeout()

クローズタイムアウトを取得します。

```rust
pub fn close_timeout(&self) -> Option<Duration>
```

#### From トレイトの実装

`TransactionType` から `TransactionOption` への変換が可能です：

```rust
let option = TransactionOption::from(TransactionType::Short);
```

### TransactionOptionSetter トレイト

`TransactionOption` の文字列設定メソッドを提供するトレイトです。

```rust
pub trait TransactionOptionSetter<T> {
    fn set_transaction_label(&mut self, transaction_label: T);
    fn set_write_preserve(&mut self, table_names: &[T]);
    fn set_inclusive_read_area(&mut self, table_names: &[T]);
    fn set_exclusive_read_area(&mut self, table_names: &[T]);
}
```

`&str` と `String` の両方に対して実装されています。

### CommitOption

コミットオプションです。

#### メソッド

##### new()

新しいインスタンスを作成します。

デフォルト値:
- `commit_type`: `CommitType::Unspecified`
- `auto_dispose`: `false`

```rust
pub fn new() -> CommitOption
```

##### set_commit_type()

コミットタイプを設定します。

```rust
pub fn set_commit_type(&mut self, commit_type: CommitType)
```

##### commit_type()

コミットタイプを取得します。

```rust
pub fn commit_type(&self) -> CommitType
```

##### set_auto_dispose()

自動破棄フラグを設定します。

```rust
pub fn set_auto_dispose(&mut self, auto_dispose: bool)
```

##### auto_dispose()

自動破棄フラグを取得します。

```rust
pub fn auto_dispose(&self) -> bool
```

#### From トレイトの実装

`CommitType` から `CommitOption` への変換が可能です：

```rust
let option = CommitOption::from(CommitType::Stored);
```

### TransactionStatusWithMessage

トランザクションステータスとメッセージを保持する型です。

**since**: 0.2.0

#### メソッド

##### status()

ステータスの列挙値を返します。

```rust
pub fn status(&self) -> TransactionStatus
```

##### message()

トランザクションステータスの追加情報を返します。

```rust
pub fn message(&self) -> &String
```

### TransactionErrorInfo

トランザクションエラー情報です。

**since**: 0.2.0

#### メソッド

##### server_error()

ターゲットトランザクションで発生したエラーを返します。トランザクションが誤ってアボートされた場合にのみ返されます。

```rust
pub fn server_error(&self) -> Option<&TgError>
```

##### is_normal()

ステータスが正常かどうかを返します。

```rust
pub fn is_normal(&self) -> bool
```

##### is_error()

ステータスがエラーかどうかを返します。

```rust
pub fn is_error(&self) -> bool
```

##### diagnostic_code()

ターゲットトランザクションでエラーが発生した場合、診断コードを返します。

```rust
pub fn diagnostic_code(&self) -> Option<&DiagnosticCode>
```

## 使用例

### OCCトランザクション

```rust
use tsubakuro_rust_core::prelude::*;

async fn example(client: &SqlClient) -> Result<(), TgError> {
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::Short);
    
    let transaction = client.start_transaction(&transaction_option).await?;
    
    // SQL実行...
    
    let commit_option = CommitOption::default();
    client.commit(&transaction, &commit_option).await?;
    
    transaction.close().await?;
    Ok(())
}
```

### LTXトランザクション

```rust
async fn example(client: &SqlClient) -> Result<(), TgError> {
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::Long);
    transaction_option.set_write_preserve(&["table1", "table2"]);
    
    let transaction = client.start_transaction(&transaction_option).await?;
    // ...
    Ok(())
}
```

### DDLトランザクション（LTX）

```rust
async fn example(client: &SqlClient) -> Result<(), TgError> {
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::Long);
    transaction_option.set_modifies_definitions(true);
    
    let transaction = client.start_transaction(&transaction_option).await?;
    // ...
    Ok(())
}
```

### RTXトランザクション

```rust
async fn example(client: &SqlClient) -> Result<(), TgError> {
    let transaction_option = TransactionOption::from(TransactionType::ReadOnly);
    
    let transaction = client.start_transaction(&transaction_option).await?;
    // ...
    Ok(())
}
```

### トランザクションステータスの取得

```rust
async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let status = client.get_transaction_status(transaction).await?;
    println!("status={:?}", status.status());
    println!("message={}", status.message());
    
    Ok(())
}
```

### トランザクションエラー情報の取得

```rust
async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let error_info = client.get_transaction_error_info(transaction).await?;
    
    if error_info.is_error() {
        if let Some(code) = error_info.diagnostic_code() {
            println!("diagnostic_code={}", code);
        }
        if let Some(error) = error_info.server_error() {
            println!("server_error={}", error);
        }
    }
    
    Ok(())
}
```

### コミットオプション

```rust
async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let mut commit_option = CommitOption::new();
    commit_option.set_commit_type(CommitType::Stored);
    commit_option.set_auto_dispose(true);
    
    client.commit(&transaction, &commit_option).await?;
    Ok(())
}
```

## 注意事項

1. **リソース管理**: `Transaction` を破棄する前に `close()` を呼び出す必要があります。`Drop` トレイトの実装では自動的に `close()` が呼び出されますが、エラーが発生する可能性があるため、明示的に呼び出すことを推奨します。

2. **トランザクションタイプ**: 
   - `Short` (OCC) - 短いトランザクション用
   - `Long` (LTX) - 長いトランザクション用、書き込み保存テーブルを指定可能
   - `ReadOnly` (RTX) - 読み取り専用トランザクション用

3. **DDLとDML**: DDLとDMLは同じトランザクション内で実行できません。別々のトランザクションを使用してください。

4. **書き込み保存**: LTXトランザクションでは、書き込みを行うテーブルを `set_write_preserve()` で事前に指定する必要があります。

## 関連モジュール

- [prelude.md](./prelude.md) - 公開API
- [service.md](./service.md) - トランザクションの開始とコミット

