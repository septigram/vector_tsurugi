# session モジュール仕様

## 概要

`session` モジュールは、Tsurugi DBサーバーへの接続とセッション管理を提供します。

## 主要な型

### Session

Tsurugiサーバーへの接続を表す型です。

**注意**: セッションを破棄する前に [`Self::close`] を呼び出す必要があります。

#### メソッド

##### connect()

Tsurugiサーバーへの接続を確立します。

```rust
pub async fn connect(connection_option: &ConnectionOption) -> Result<Arc<Session>, TgError>
```

**パラメータ**:
- `connection_option` - 接続オプション

**戻り値**: `Arc<Session>` - 確立されたセッション

**注意**: セッションを破棄する前に [`Self::close`] を呼び出す必要があります。

##### connect_for()

タイムアウトを指定してTsurugiサーバーへの接続を確立します。

```rust
pub async fn connect_for(
    connection_option: &ConnectionOption,
    timeout: Duration,
) -> Result<Arc<Session>, TgError>
```

##### connect_async()

非同期でTsurugiサーバーへの接続を確立します。

```rust
pub async fn connect_async(
    connection_option: &ConnectionOption,
) -> Result<Job<Arc<Session>>, TgError>
```

##### user_name()

ユーザー名を取得します。

```rust
pub fn user_name(&self) -> Option<String>
```

**since**: 0.5.0

##### has_encryption_key()

セッションが暗号化キーを持っているかどうかを確認します。

```rust
pub async fn has_encryption_key(&self) -> bool
```

**since**: 0.5.0

**注意**: 内部使用向け

##### set_default_timeout()

デフォルトタイムアウトを設定します。

```rust
pub fn set_default_timeout(&self, timeout: Duration)
```

##### default_timeout()

デフォルトタイムアウトを取得します。

```rust
pub fn default_timeout(&self) -> Duration
```

##### make_client()

サービスクライアントを作成します。

```rust
pub fn make_client<T: ServiceClient>(self: &Arc<Session>) -> T
```

**例**:
```rust
let client: SqlClient = session.make_client();
```

##### update_expiration_time()

セッションの有効期限を更新します。

リソースはセッションの有効期限後に破棄されます。有効期限を延長するには、セッション内でリクエストを継続的に送信するか、このメソッドを使用して明示的に有効期限を更新します。

指定された有効期限が長すぎる場合、サーバーは自動的に上限に短縮します。また、時間が短すぎる場合や0未満の場合、サーバーはリクエストを無視することがあります。

```rust
pub async fn update_expiration_time(
    &self,
    expiration_time: Option<Duration>,
) -> Result<(), TgError>
```

##### update_expiration_time_for()

タイムアウトを指定してセッションの有効期限を更新します。

```rust
pub async fn update_expiration_time_for(
    &self,
    expiration_time: Option<Duration>,
    timeout: Duration,
) -> Result<(), TgError>
```

##### update_expiration_time_async()

非同期でセッションの有効期限を更新します。

```rust
pub async fn update_expiration_time_async(
    &self,
    expiration_time: Option<Duration>,
) -> Result<Job<()>, TgError>
```

##### shutdown()

現在のセッションをシャットダウンし、実行中のリクエストの完了を待機します。

```rust
pub async fn shutdown(&self, shutdown_type: ShutdownType) -> Result<(), TgError>
```

##### shutdown_for()

タイムアウトを指定してセッションをシャットダウンします。

```rust
pub async fn shutdown_for(
    &self,
    shutdown_type: ShutdownType,
    timeout: Duration,
) -> Result<(), TgError>
```

##### shutdown_async()

非同期でセッションをシャットダウンします。

```rust
pub async fn shutdown_async(&self, shutdown_type: ShutdownType) -> Result<Job<()>, TgError>
```

##### is_shutdowned()

セッションがシャットダウンされているかどうかを確認します。

```rust
pub fn is_shutdowned(&self) -> bool
```

##### close()

現在のセッションを破棄します。

実行中のリクエストの完了を待たない場合があります。安全にセッションを閉じるには [`Self::shutdown`] を使用してください。

**注意**: セッションを破棄する前に `close` を呼び出す必要があります。

```rust
pub async fn close(&self) -> Result<(), TgError>
```

##### is_closed()

セッションが閉じられているかどうかを確認します。

```rust
pub fn is_closed(&self) -> bool
```

### ConnectionOption

Tsurugiサーバーへの接続オプションです。

#### メソッド

##### new()

新しいインスタンスを作成します。

```rust
pub fn new() -> ConnectionOption
```

##### set_endpoint()

エンドポイントを設定します。

```rust
pub fn set_endpoint(&mut self, endpoint: Endpoint)
```

##### set_endpoint_url()

エンドポイントURLを設定します。

```rust
pub fn set_endpoint_url(&mut self, endpoint: &str) -> Result<(), TgError>
```

**パラメータ**:
- `endpoint` - エンドポイントURL（例: `tcp://localhost:12345`）

##### endpoint()

エンドポイントを取得します。

```rust
pub fn endpoint(&self) -> Option<&Endpoint>
```

##### set_credential()

認証情報を設定します。

```rust
pub fn set_credential(&mut self, credential: Credential)
```

**since**: 0.5.0

##### credential()

認証情報を取得します。

```rust
pub fn credential(&self) -> &Credential
```

**since**: 0.5.0

##### set_validity_period()

UserPasswordCredentialの有効期間を設定します。

**注意**: 内部使用向け

```rust
pub fn set_validity_period(&mut self, duration: Duration)
```

**since**: 0.5.0

##### validity_period()

有効期間を取得します。

```rust
pub fn validity_period(&self) -> Duration
```

**since**: 0.5.0

##### set_application_name()

アプリケーション名を設定します。

```rust
pub fn set_application_name(&mut self, name: &str)
```

##### application_name()

アプリケーション名を取得します。

```rust
pub fn application_name(&self) -> Option<&String>
```

##### set_session_label()

セッションラベルを設定します。

```rust
pub fn set_session_label(&mut self, label: &str)
```

##### session_label()

セッションラベルを取得します。

```rust
pub fn session_label(&self) -> Option<&String>
```

##### set_keep_alive()

キープアライブ間隔を設定します。

`keep_alive` が 0 の場合、キープアライブを実行しません。

```rust
pub fn set_keep_alive(&mut self, keep_alive: Duration)
```

##### keep_alive()

キープアライブ間隔を取得します。

```rust
pub fn keep_alive(&self) -> Duration
```

##### add_large_object_path_mapping()

BLOB/CLOBの送受信両方のパスマッピングエントリを追加します。

```rust
pub fn add_large_object_path_mapping<T: AsRef<Path>>(
    &mut self,
    client_path: T,
    server_path: &str,
)
```

**since**: 0.2.0

##### add_large_object_path_mapping_on_send()

BLOB/CLOB送信時のパスマッピングエントリを追加します。

```rust
pub fn add_large_object_path_mapping_on_send<T: AsRef<Path>>(
    &mut self,
    client_path: T,
    server_path: &str,
)
```

**since**: 0.2.0

##### add_large_object_path_mapping_on_recv()

BLOB/CLOB受信時のパスマッピングエントリを追加します。

```rust
pub fn add_large_object_path_mapping_on_recv<T: AsRef<Path>>(
    &mut self,
    server_path: &str,
    client_path: T,
)
```

**since**: 0.2.0

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

##### set_send_timeout()

通信送信タイムアウトを設定します。

```rust
pub fn set_send_timeout(&mut self, timeout: Duration)
```

##### send_timeout()

通信送信タイムアウトを取得します。

```rust
pub fn send_timeout(&self) -> Duration
```

##### set_recv_timeout()

通信受信タイムアウトを設定します。

```rust
pub fn set_recv_timeout(&mut self, timeout: Duration)
```

##### recv_timeout()

通信受信タイムアウトを取得します。

```rust
pub fn recv_timeout(&self) -> Duration
```

### Credential

認証情報を表す列挙型です。

**since**: 0.5.0

```rust
pub enum Credential {
    Null,
    UserPassword {
        user: String,
        password: Option<String>,
    },
    AuthToken(String),
    File {
        encrypted: String,
        comments: Vec<String>,
    },
}
```

#### メソッド

##### null()

null認証情報を返します。

```rust
pub fn null() -> Credential
```

##### from_user_password()

ユーザー/パスワード認証情報を作成します。

```rust
pub fn from_user_password(
    user: impl Into<String>,
    password: Option<impl Into<String>>,
) -> Credential
```

##### from_auth_token()

認証トークン認証情報を作成します。

```rust
pub fn from_auth_token(token: impl Into<String>) -> Credential
```

##### load()

ファイルから認証情報を読み込みます。

```rust
pub fn load(path: impl AsRef<Path>) -> Result<Credential, TgError>
```

### Endpoint

エンドポイントを表す列挙型です。

```rust
pub enum Endpoint {
    Tcp(String, u16),  // host, port
    Other,             // ダミー（非公開）
}
```

#### メソッド

##### parse()

エンドポイントURLをパースします。

```rust
pub fn parse(endpoint: &str) -> Result<Endpoint, TgError>
```

**パラメータ**:
- `endpoint` - エンドポイントURL（例: `tcp://localhost:12345`）

## 使用例

### 基本的な接続

```rust
use std::time::Duration;
use tsubakuro_rust_core::prelude::*;

async fn example() -> Result<(), TgError> {
    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url("tcp://localhost:12345")?;
    connection_option.set_application_name("My App");
    connection_option.set_session_label("example session");
    connection_option.set_default_timeout(Duration::from_secs(10));

    let session = Session::connect(&connection_option).await?;
    let client: SqlClient = session.make_client();

    // 使用...

    session.close().await?;
    Ok(())
}
```

### 認証情報を使用した接続

```rust
async fn example() -> Result<(), TgError> {
    let credential = Credential::from_user_password("user", Some("password"));

    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url("tcp://localhost:12345")?;
    connection_option.set_credential(credential);
    connection_option.set_default_timeout(Duration::from_secs(10));

    let session = Session::connect(&connection_option).await?;
    // ...
    Ok(())
}
```

### 非同期接続

```rust
async fn example() -> Result<(), TgError> {
    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url("tcp://localhost:12345")?;
    connection_option.set_default_timeout(Duration::from_secs(10));

    let mut job = Session::connect_async(&connection_option).await?;
    let session = job.take().await?;
    // ...
    Ok(())
}
```

## 注意事項

1. **リソース管理**: `Session` を破棄する前に `close()` を呼び出す必要があります。`Drop` トレイトの実装では自動的に `close()` が呼び出されますが、エラーが発生する可能性があるため、明示的に呼び出すことを推奨します。

2. **スレッド安全性**: `Session` は `Arc` で包まれて返されるため、複数のスレッド間で共有できます。

3. **キープアライブ**: `keep_alive` が設定されている場合、バックグラウンドで自動的に有効期限が更新されます。

4. **エンドポイント**: 現在はTCPエンドポイントのみサポートされています。

## 関連モジュール

- [prelude.md](./prelude.md) - 公開API
- [service.md](./service.md) - サービスクライアントの作成

