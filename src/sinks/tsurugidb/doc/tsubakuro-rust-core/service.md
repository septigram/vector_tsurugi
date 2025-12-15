# service モジュール仕様

## 概要

`service` モジュールは、Tsurugi DBのサービス（特にSQLサービス）へのアクセスを提供します。

## 主要な型とトレイト

### ServiceClient トレイト

サービスクライアントを表すトレイトです。

```rust
pub trait ServiceClient {
    fn new(session: Arc<Session>) -> Self;
}
```

#### 実装型

- `SqlClient` - SQLサービスクライアント

### ServiceMessageVersion トレイト

サービスメッセージバージョンを提供するトレイトです。

**since**: 0.7.0

```rust
pub trait ServiceMessageVersion {
    fn service_message_version() -> String;
}
```

### SqlClient

SQLサービスクライアントです。

#### メソッド

##### タイムアウト管理

##### set_default_timeout()

デフォルトタイムアウトを設定します。

```rust
pub fn set_default_timeout(&mut self, timeout: Duration)
```

##### default_timeout()

デフォルトタイムアウトを取得します。

```rust
pub fn default_timeout(&self) -> Duration
```

##### テーブル情報

##### list_tables()

データベース内の利用可能なテーブル名のリストを返します（システムテーブルを除く）。

テーブル名は完全修飾名（スキーマ名を含む場合がある）です。個別のテーブルの詳細情報を取得するには [`Self::get_table_metadata`] を使用してください。

```rust
pub async fn list_tables(&self) -> Result<TableList, TgError>
```

##### list_tables_for()

タイムアウトを指定してテーブルリストを取得します。

```rust
pub async fn list_tables_for(&self, timeout: Duration) -> Result<TableList, TgError>
```

##### list_tables_async()

非同期でテーブルリストを取得します。

```rust
pub async fn list_tables_async(&self) -> Result<Job<TableList>, TgError>
```

##### get_table_metadata()

テーブルのメタデータを取得します。

```rust
pub async fn get_table_metadata(&self, table_name: &str) -> Result<TableMetadata, TgError>
```

##### get_table_metadata_for()

タイムアウトを指定してテーブルメタデータを取得します。

```rust
pub async fn get_table_metadata_for(
    &self,
    table_name: &str,
    timeout: Duration,
) -> Result<TableMetadata, TgError>
```

##### get_table_metadata_async()

非同期でテーブルメタデータを取得します。

```rust
pub async fn get_table_metadata_async(
    &self,
    table_name: &str,
) -> Result<Job<TableMetadata>, TgError>
```

##### プリペアドステートメント

##### prepare()

SQLステートメントを準備します。

**注意**: プリペアドステートメントを破棄する前に [`SqlPreparedStatement::close`] を呼び出す必要があります。

```rust
pub async fn prepare(
    &self,
    sql: &str,
    placeholders: Vec<SqlPlaceholder>,
) -> Result<SqlPreparedStatement, TgError>
```

##### prepare_for()

タイムアウトを指定してSQLステートメントを準備します。

```rust
pub async fn prepare_for(
    &self,
    sql: &str,
    placeholders: Vec<SqlPlaceholder>,
    timeout: Duration,
) -> Result<SqlPreparedStatement, TgError>
```

##### prepare_async()

非同期でSQLステートメントを準備します。

```rust
pub async fn prepare_async(
    &self,
    sql: &str,
    placeholders: Vec<SqlPlaceholder>,
) -> Result<Job<SqlPreparedStatement>, TgError>
```

##### 実行計画（EXPLAIN）

##### explain()

ステートメントの実行計画を取得します。

```rust
pub async fn explain(&self, sql: &str) -> Result<SqlExplainResult, TgError>
```

##### explain_for()

タイムアウトを指定して実行計画を取得します。

```rust
pub async fn explain_for(
    &self,
    sql: &str,
    timeout: Duration,
) -> Result<SqlExplainResult, TgError>
```

##### explain_async()

非同期で実行計画を取得します。

```rust
pub async fn explain_async(&self, sql: &str) -> Result<Job<SqlExplainResult>, TgError>
```

##### prepared_explain()

プリペアドステートメントの実行計画を取得します。

```rust
pub async fn prepared_explain(
    &self,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<SqlExplainResult, TgError>
```

##### prepared_explain_for()

タイムアウトを指定してプリペアドステートメントの実行計画を取得します。

```rust
pub async fn prepared_explain_for(
    &self,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
    timeout: Duration,
) -> Result<SqlExplainResult, TgError>
```

##### prepared_explain_async()

非同期でプリペアドステートメントの実行計画を取得します。

```rust
pub async fn prepared_explain_async(
    &self,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<Job<SqlExplainResult>, TgError>
```

##### トランザクション管理

##### start_transaction()

新しいトランザクションを開始します。

**注意**: トランザクションを破棄する前に [`Transaction::close`] を呼び出す必要があります。

```rust
pub async fn start_transaction(
    &self,
    transaction_option: &TransactionOption,
) -> Result<Transaction, TgError>
```

##### start_transaction_for()

タイムアウトを指定してトランザクションを開始します。

```rust
pub async fn start_transaction_for(
    &self,
    transaction_option: &TransactionOption,
    timeout: Duration,
) -> Result<Transaction, TgError>
```

##### start_transaction_async()

非同期でトランザクションを開始します。

```rust
pub async fn start_transaction_async(
    &self,
    transaction_option: &TransactionOption,
) -> Result<Job<Transaction>, TgError>
```

##### get_transaction_status()

サーバー上のトランザクションステータスを取得します。

**since**: 0.2.0

```rust
pub async fn get_transaction_status(
    &self,
    transaction: &Transaction,
) -> Result<TransactionStatusWithMessage, TgError>
```

##### get_transaction_status_for()

タイムアウトを指定してトランザクションステータスを取得します。

```rust
pub async fn get_transaction_status_for(
    &self,
    transaction: &Transaction,
    timeout: Duration,
) -> Result<TransactionStatusWithMessage, TgError>
```

##### get_transaction_status_async()

非同期でトランザクションステータスを取得します。

```rust
pub async fn get_transaction_status_async(
    &self,
    transaction: &Transaction,
) -> Result<Job<TransactionStatusWithMessage>, TgError>
```

##### get_transaction_error_info()

ターゲットトランザクションで発生したエラーを取得します。

**since**: 0.2.0

```rust
pub async fn get_transaction_error_info(
    &self,
    transaction: &Transaction,
) -> Result<TransactionErrorInfo, TgError>
```

##### get_transaction_error_info_for()

タイムアウトを指定してトランザクションエラー情報を取得します。

```rust
pub async fn get_transaction_error_info_for(
    &self,
    transaction: &Transaction,
    timeout: Duration,
) -> Result<TransactionErrorInfo, TgError>
```

##### get_transaction_error_info_async()

非同期でトランザクションエラー情報を取得します。

```rust
pub async fn get_transaction_error_info_async(
    &self,
    transaction: &Transaction,
) -> Result<Job<TransactionErrorInfo>, TgError>
```

##### SQL実行

##### execute()

SQLステートメントを実行します。

```rust
pub async fn execute(
    &self,
    transaction: &Transaction,
    sql: &str,
) -> Result<SqlExecuteResult, TgError>
```

##### execute_for()

タイムアウトを指定してSQLステートメントを実行します。

```rust
pub async fn execute_for(
    &self,
    transaction: &Transaction,
    sql: &str,
    timeout: Duration,
) -> Result<SqlExecuteResult, TgError>
```

##### execute_async()

非同期でSQLステートメントを実行します。

```rust
pub async fn execute_async(
    &self,
    transaction: &Transaction,
    sql: &str,
) -> Result<Job<SqlExecuteResult>, TgError>
```

##### prepared_execute()

プリペアドステートメントを実行します。

```rust
pub async fn prepared_execute(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<SqlExecuteResult, TgError>
```

##### prepared_execute_for()

タイムアウトを指定してプリペアドステートメントを実行します。

```rust
pub async fn prepared_execute_for(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
    timeout: Duration,
) -> Result<SqlExecuteResult, TgError>
```

##### prepared_execute_async()

非同期でプリペアドステートメントを実行します。

```rust
pub async fn prepared_execute_async(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<Job<SqlExecuteResult>, TgError>
```

##### クエリ実行

##### query()

SQLステートメントを実行し、その結果を取得します。

```rust
pub async fn query(
    &self,
    transaction: &Transaction,
    sql: &str,
) -> Result<SqlQueryResult, TgError>
```

##### query_for()

タイムアウトを指定してSQLクエリを実行します。

```rust
pub async fn query_for(
    &self,
    transaction: &Transaction,
    sql: &str,
    timeout: Duration,
) -> Result<SqlQueryResult, TgError>
```

##### query_async()

非同期でSQLクエリを実行します。

```rust
pub async fn query_async(
    &self,
    transaction: &Transaction,
    sql: &str,
) -> Result<Job<SqlQueryResult>, TgError>
```

##### prepared_query()

プリペアドステートメントを実行し、その結果を取得します。

```rust
pub async fn prepared_query(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<SqlQueryResult, TgError>
```

##### prepared_query_for()

タイムアウトを指定してプリペアドステートメントのクエリを実行します。

```rust
pub async fn prepared_query_for(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
    timeout: Duration,
) -> Result<SqlQueryResult, TgError>
```

##### prepared_query_async()

非同期でプリペアドステートメントのクエリを実行します。

```rust
pub async fn prepared_query_async(
    &self,
    transaction: &Transaction,
    prepared_statement: &SqlPreparedStatement,
    parameters: Vec<SqlParameter>,
) -> Result<Job<SqlQueryResult>, TgError>
```

##### コミット/ロールバック

##### commit()

SQLサービスにコミットを要求します。

```rust
pub async fn commit(
    &self,
    transaction: &Transaction,
    commit_option: &CommitOption,
) -> Result<(), TgError>
```

##### commit_for()

タイムアウトを指定してコミットを要求します。

```rust
pub async fn commit_for(
    &self,
    transaction: &Transaction,
    commit_option: &CommitOption,
    timeout: Duration,
) -> Result<(), TgError>
```

##### commit_async()

非同期でコミットを要求します。

```rust
pub async fn commit_async(
    &self,
    transaction: &Transaction,
    commit_option: &CommitOption,
) -> Result<Job<()>, TgError>
```

##### rollback()

SQLサービスにロールバックを要求します。

```rust
pub async fn rollback(&self, transaction: &Transaction) -> Result<(), TgError>
```

##### rollback_for()

タイムアウトを指定してロールバックを要求します。

```rust
pub async fn rollback_for(
    &self,
    transaction: &Transaction,
    timeout: Duration,
) -> Result<(), TgError>
```

##### rollback_async()

非同期でロールバックを要求します。

```rust
pub async fn rollback_async(&self, transaction: &Transaction) -> Result<Job<()>, TgError>
```

##### ラージオブジェクト（BLOB/CLOB）

##### open_blob()

BLOBファイルを開きます。

```rust
pub async fn open_blob(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<std::fs::File, TgError>
```

##### open_blob_for()

タイムアウトを指定してBLOBファイルを開きます。

```rust
pub async fn open_blob_for(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    timeout: Duration,
) -> Result<std::fs::File, TgError>
```

##### open_blob_async()

非同期でBLOBファイルを開きます。

```rust
pub async fn open_blob_async(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<Job<std::fs::File>, TgError>
```

##### open_clob()

CLOBファイルを開きます。

```rust
pub async fn open_clob(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<std::fs::File, TgError>
```

##### open_clob_for()

タイムアウトを指定してCLOBファイルを開きます。

```rust
pub async fn open_clob_for(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    timeout: Duration,
) -> Result<std::fs::File, TgError>
```

##### open_clob_async()

非同期でCLOBファイルを開きます。

```rust
pub async fn open_clob_async(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<Job<std::fs::File>, TgError>
```

##### get_blob_cache()

BLOBキャッシュを取得します。

**since**: 0.5.0

```rust
pub async fn get_blob_cache(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<TgLargeObjectCache, TgError>
```

##### get_blob_cache_for()

タイムアウトを指定してBLOBキャッシュを取得します。

```rust
pub async fn get_blob_cache_for(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    timeout: Duration,
) -> Result<TgLargeObjectCache, TgError>
```

##### get_blob_cache_async()

非同期でBLOBキャッシュを取得します。

```rust
pub async fn get_blob_cache_async(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<Job<TgLargeObjectCache>, TgError>
```

##### get_clob_cache()

CLOBキャッシュを取得します。

**since**: 0.5.0

```rust
pub async fn get_clob_cache(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<TgLargeObjectCache, TgError>
```

##### get_clob_cache_for()

タイムアウトを指定してCLOBキャッシュを取得します。

```rust
pub async fn get_clob_cache_for(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    timeout: Duration,
) -> Result<TgLargeObjectCache, TgError>
```

##### get_clob_cache_async()

非同期でCLOBキャッシュを取得します。

```rust
pub async fn get_clob_cache_async(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<Job<TgLargeObjectCache>, TgError>
```

##### read_blob()

BLOBを読み込みます。

**since**: 0.2.0

```rust
pub async fn read_blob(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<Vec<u8>, TgError>
```

##### read_blob_for()

タイムアウトを指定してBLOBを読み込みます。

```rust
pub async fn read_blob_for(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    timeout: Duration,
) -> Result<Vec<u8>, TgError>
```

##### read_blob_async()

非同期でBLOBを読み込みます。

```rust
pub async fn read_blob_async(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
) -> Result<Job<Vec<u8>>, TgError>
```

##### read_clob()

CLOBを読み込みます。

**since**: 0.2.0

```rust
pub async fn read_clob(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<String, TgError>
```

##### read_clob_for()

タイムアウトを指定してCLOBを読み込みます。

```rust
pub async fn read_clob_for(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    timeout: Duration,
) -> Result<String, TgError>
```

##### read_clob_async()

非同期でCLOBを読み込みます。

```rust
pub async fn read_clob_async(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
) -> Result<Job<String>, TgError>
```

##### copy_blob_to()

BLOBをローカルファイルにコピーします。

```rust
pub async fn copy_blob_to<T: AsRef<Path>>(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    destination: T,
) -> Result<(), TgError>
```

##### copy_blob_to_for()

タイムアウトを指定してBLOBをローカルファイルにコピーします。

```rust
pub async fn copy_blob_to_for<T: AsRef<Path>>(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    destination: T,
    timeout: Duration,
) -> Result<(), TgError>
```

##### copy_blob_to_async()

非同期でBLOBをローカルファイルにコピーします。

```rust
pub async fn copy_blob_to_async<T: AsRef<Path> + Send + Clone + 'static>(
    &self,
    transaction: &Transaction,
    blob: &TgBlobReference,
    destination: T,
) -> Result<Job<()>, TgError>
```

##### copy_clob_to()

CLOBをローカルファイルにコピーします。

```rust
pub async fn copy_clob_to<T: AsRef<Path>>(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    destination: T,
) -> Result<(), TgError>
```

##### copy_clob_to_for()

タイムアウトを指定してCLOBをローカルファイルにコピーします。

```rust
pub async fn copy_clob_to_for<T: AsRef<Path>>(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    destination: T,
    timeout: Duration,
) -> Result<(), TgError>
```

##### copy_clob_to_async()

非同期でCLOBをローカルファイルにコピーします。

```rust
pub async fn copy_clob_to_async<T: AsRef<Path> + Send + Clone + 'static>(
    &self,
    transaction: &Transaction,
    clob: &TgClobReference,
    destination: T,
) -> Result<Job<()>, TgError>
```

## 結果型

### SqlExecuteResult

SQL実行結果を表す型です。

#### メソッド

##### counters()

この結果で利用可能なすべてのカウンターエントリを返します。

```rust
pub fn counters(&self) -> &HashMap<CounterType, i64>
```

##### inserted_rows()

挿入された行数を取得します。

```rust
pub fn inserted_rows(&self) -> i64
```

##### updated_rows()

更新された行数を取得します。

```rust
pub fn updated_rows(&self) -> i64
```

##### merged_rows()

マージされた行数を取得します。

```rust
pub fn merged_rows(&self) -> i64
```

##### deleted_rows()

削除された行数を取得します。

```rust
pub fn deleted_rows(&self) -> i64
```

##### rows()

総行数を取得します。

```rust
pub fn rows(&self) -> i64
```

### SqlQueryResult

SQLクエリ結果を表す型です。

トランザクションが生きている間のみ使用できます。

**スレッド安全性**: スレッド非安全（thread unsafe）

#### メソッド

##### set_default_timeout()

デフォルトタイムアウトを設定します。

```rust
pub fn set_default_timeout(&mut self, timeout: Duration)
```

##### default_timeout()

デフォルトタイムアウトを取得します。

```rust
pub fn default_timeout(&mut self) -> &Duration
```

##### get_metadata()

このクエリ結果のメタデータを返します。

```rust
pub fn get_metadata(&self) -> Option<&SqlQueryResultMetadata>
```

##### next_row()

次の行に移動します。

```rust
pub async fn next_row(&mut self) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - 行が存在
- `Ok(false)` - 行が存在しない

##### next_column()

次のカラムに移動します。

```rust
pub async fn next_column(&mut self) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - カラムが存在
- `Ok(false)` - カラムが存在しない

##### is_null()

現在のカラムがnullかどうかを確認します。

```rust
pub fn is_null(&self) -> Result<bool, TgError>
```

##### fetch()

現在のカラムの値を取得します。

```rust
pub async fn fetch<T>(&mut self) -> Result<T, TgError>
where
    T: SqlQueryResultFetch,
```

サポートされる型については、`SqlQueryResultFetch` トレイトの実装を参照してください。

##### close()

リソースを破棄します。

```rust
pub async fn close(&mut self) -> Result<(), TgError>
```

### SqlPreparedStatement

プリペアドステートメントを表す型です。

**注意**: プリペアドステートメントを破棄する前に [`Self::close`] を呼び出す必要があります。

#### メソッド

##### has_result_records()

このステートメントを実行した結果としてResultRecordsが返されるかどうかを確認します。

```rust
pub fn has_result_records(&self) -> bool
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

```rust
pub async fn close(&self) -> Result<(), TgError>
```

##### close_for()

タイムアウトを指定してリソースを破棄します。

```rust
pub async fn close_for(&self, timeout: Duration) -> Result<(), TgError>
```

### SqlExplainResult

SQL実行計画結果を表す型です。

#### メソッド

##### format_id()

コンテンツフォーマットIDを返します。

```rust
pub fn format_id(&self) -> &String
```

##### format_version()

コンテンツフォーマットバージョンを返します。

```rust
pub fn format_version(&self) -> u64
```

##### contents()

実行計画結果の内容を返します。

```rust
pub fn contents(&self) -> &String
```

##### columns()

カラム情報を返します。提供されていない場合は空です。

```rust
pub fn columns(&self) -> &Vec<SqlColumn>
```

### TableList

テーブルリストを表す型です。

#### メソッド

##### table_names()

データベース内の利用可能なテーブル名のリストを返します（システムテーブルを除く）。

```rust
pub fn table_names(&self) -> &Vec<TName>
```

### TableMetadata

テーブルメタデータを表す型です。

#### メソッド

##### database_name()

テーブルが定義されているデータベース名を返します。

```rust
pub fn database_name(&self) -> &String
```

##### schema_name()

テーブルが定義されているスキーマ名を返します。

```rust
pub fn schema_name(&self) -> &String
```

##### table_name()

テーブルのシンプル名を返します。

```rust
pub fn table_name(&self) -> &String
```

##### description()

テーブルの説明を返します。

**since**: 0.2.0

```rust
pub fn description(&self) -> Option<&String>
```

##### columns()

テーブルのカラム情報を返します。

```rust
pub fn columns(&self) -> &Vec<SqlColumn>
```

##### primary_keys()

テーブルの主キーを返します。

**since**: 0.3.0

```rust
pub fn primary_keys(&self) -> &Vec<String>
```

## パラメータとプレースホルダ

### SqlParameter

SQLパラメータを表す型です。

#### メソッド

##### null()

nullパラメータを作成します。

```rust
pub fn null(name: &str) -> SqlParameter
```

##### name()

名前を取得します。

```rust
pub fn name(&self) -> Option<&String>
```

##### value()

値を取得します。

```rust
pub fn value(&self) -> Option<&Value>
```

#### SqlParameterOf トレイト

型指定でパラメータを作成するためのトレイトです。

```rust
pub trait SqlParameterOf<T> {
    fn of(name: &str, value: T) -> SqlParameter;
}
```

サポートされる型: `bool`, `i32`, `i64`, `f32`, `f64`, `TgDecimal`, `TgDecimalI128`, `String`, `TgDate`, `TgTimeOfDay`, `TgTimePoint`, など

### SqlPlaceholder

SQLプレースホルダを表す型です。

#### メソッド

##### of_atom_type()

AtomTypeを指定してインスタンスを作成します。

```rust
pub fn of_atom_type(name: &str, atom_type: AtomType) -> SqlPlaceholder
```

##### of()

型指定でインスタンスを作成します。

```rust
pub fn of<T: AtomTypeProvider>(name: &str) -> Self
```

##### name()

名前を取得します。

```rust
pub fn name(&self) -> Option<&String>
```

##### atom_type()

AtomTypeを取得します。

```rust
pub fn atom_type(&self) -> Option<AtomType>
```

#### AtomTypeProvider トレイト

AtomTypeを提供するトレイトです。

```rust
pub trait AtomTypeProvider {
    fn atom_type() -> AtomType;
}
```

サポートされる型: `bool`, `i32`, `i64`, `f32`, `f64`, `TgDecimal`, `TgDecimalI128`, `String`, `TgDate`, `TgTimeOfDay`, `TgTimePoint`, など

## 使用例

### 基本的なSQL実行

```rust
use tsubakuro_rust_core::prelude::*;

async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let sql = "insert into customer values(1, 'Alice', 25)";
    let execute_result = client.execute(transaction, sql).await?;
    println!("inserted rows={}", execute_result.inserted_rows());
    
    Ok(())
}
```

### クエリの実行

```rust
async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let sql = "select c_id, c_name, c_age from customer order by c_id";
    let mut query_result = client.query(transaction, sql).await?;
    
    while query_result.next_row().await? {
        if query_result.next_column().await? {
            let id: i64 = query_result.fetch().await?;
        }
        if query_result.next_column().await? {
            let name: Option<String> = query_result.fetch().await?;
        }
        if query_result.next_column().await? {
            let age: Option<i32> = query_result.fetch().await?;
        }
    }
    
    query_result.close().await?;
    Ok(())
}
```

### プリペアドステートメント

```rust
async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let sql = "insert into customer values(:id, :name, :age)";
    let placeholders = vec![
        SqlPlaceholder::of::<i64>("id"),
        SqlPlaceholder::of::<String>("name"),
        SqlPlaceholder::of::<i32>("age"),
    ];
    let prepared_statement = client.prepare(sql, placeholders).await?;
    
    let parameters = vec![
        SqlParameter::of("id", 1_i64),
        SqlParameter::of("name", "Alice"),
        SqlParameter::of("age", 25),
    ];
    let execute_result = client.prepared_execute(transaction, &prepared_statement, parameters).await?;
    
    prepared_statement.close().await?;
    Ok(())
}
```

### テーブル情報の取得

```rust
async fn example(client: &SqlClient) -> Result<(), TgError> {
    let table_list = client.list_tables().await?;
    for table_name in table_list.table_names() {
        println!("table: {}", table_name);
    }
    
    let table_metadata = client.get_table_metadata("customer").await?;
    println!("table name={}", table_metadata.table_name());
    for column in table_metadata.columns() {
        println!("column name={}", column.name());
    }
    
    Ok(())
}
```

### 実行計画の取得

```rust
async fn example(client: &SqlClient) -> Result<(), TgError> {
    let sql = "select * from customer order by c_id";
    let explain_result = client.explain(sql).await?;
    println!("json={}", explain_result.contents());
    
    Ok(())
}
```

### BLOB/CLOBの操作

```rust
use std::io::Read;

async fn example(client: &SqlClient, transaction: &Transaction, query_result: &mut SqlQueryResult) -> Result<(), TgError> {
    let blob: TgBlobReference = query_result.fetch().await?;
    
    // BLOBファイルを開く
    let mut file = client.open_blob(transaction, &blob).await?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // または、直接読み込む
    let bytes = client.read_blob(transaction, &blob).await?;
    
    // または、ファイルにコピー
    client.copy_blob_to(transaction, &blob, "/path/to/blob.bin").await?;
    
    Ok(())
}
```

## 注意事項

1. **リソース管理**: `SqlPreparedStatement` と `SqlQueryResult` を破棄する前に `close()` を呼び出す必要があります。

2. **スレッド安全性**: `SqlQueryResult` はスレッド非安全です。

3. **トランザクションの有効期限**: `SqlQueryResult` は、トランザクションが生きている間のみ使用できます。

4. **非同期API**: すべての主要なAPIには、`_async` サフィックス付きの非同期バージョンが用意されています。これらは `Job<T>` を返し、バックグラウンドで処理を実行できます。

5. **機能フラグ**: 一部のデータ型（日時・数値型）は、対応する機能フラグが有効な場合のみ使用できます。

## 関連モジュール

- [prelude.md](./prelude.md) - 公開API
- [transaction.md](./transaction.md) - トランザクション管理
- [session.md](./session.md) - セッション管理
- [job.md](./job.md) - 非同期ジョブ

