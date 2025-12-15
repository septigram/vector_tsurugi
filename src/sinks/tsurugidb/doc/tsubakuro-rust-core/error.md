# error モジュール仕様

## 概要

`error` モジュールは、tsubakuro-rust-core のエラー処理を提供します。

このモジュールは非公開（`#[doc(hidden)]`）ですが、`prelude` モジュールを通じて `TgError` 型が公開されています。

## 主要な型

### TgError

`TgError` は、このライブラリで使用されるすべてのエラーを表現する列挙型です。

```rust
pub enum TgError {
    /// クライアントエラー
    ClientError(String, Option<Box<dyn std::error::Error>>),
    
    /// タイムアウトエラー
    TimeoutError(String),
    
    /// I/Oエラー
    IoError(String, Option<Box<dyn std::error::Error>>),
    
    /// サーバーエラー
    ServerError(String, String, DiagnosticCode, String),
}
```

#### バリアント

##### ClientError

クライアント側で発生したエラーを表します。

- **第1引数**: エラーメッセージ
- **第2引数**: 原因となったエラー（オプション）

##### TimeoutError

タイムアウトが発生したことを表します。

- **引数**: エラーメッセージ

##### IoError

I/O操作でエラーが発生したことを表します。

- **第1引数**: エラーメッセージ
- **第2引数**: 原因となったI/Oエラー（オプション）

##### ServerError

サーバー側でエラーが発生したことを表します。

- **第1引数**: 関数名
- **第2引数**: エラーメッセージ
- **第3引数**: 診断コード (`DiagnosticCode`)
- **第4引数**: サーバーメッセージ

#### メソッド

##### message()

エラーメッセージを取得します。

```rust
pub fn message(&self) -> &String
```

##### diagnostic_code()

サーバーエラーの診断コードを取得します。`ServerError` の場合のみ `Some` を返し、それ以外は `None` を返します。

```rust
pub fn diagnostic_code(&self) -> Option<&DiagnosticCode>
```

#### Error トレイトの実装

`TgError` は `std::error::Error` トレイトを実装しており、`source()` メソッドで原因となったエラーを取得できます。

### DiagnosticCode

サーバーエラーの診断コードを表す構造体です。

```rust
pub struct DiagnosticCode {
    category_number: i32,
    category_str: String,
    code_number: i32,
    name: String,
}
```

#### メソッド

##### category_number()

エラーカテゴリ番号を取得します。

```rust
pub fn category_number(&self) -> i32
```

##### category_str()

エラーカテゴリ文字列を取得します。

```rust
pub fn category_str(&self) -> &String
```

##### code_number()

エラーコード番号を取得します。

```rust
pub fn code_number(&self) -> i32
```

##### structured_code()

構造化されたエラーコード（`"CATEGORY-#####"` 形式）を取得します。

```rust
pub fn structured_code(&self) -> String
```

##### name()

エラー名を取得します。

```rust
pub fn name(&self) -> &String
```

#### Display トレイトの実装

`DiagnosticCode` は `Display` トレイトを実装しており、`structured_code()` と `name()` を含む文字列を返します。

例: `"TST-00456 (TEST_EXCEPTION)"`

## 内部マクロ

このモジュールでは、エラー作成を簡素化するための内部マクロが定義されています：

- `client_error!()` - クライアントエラーを作成
- `illegal_argument_error!()` - 不正引数エラーを作成
- `io_error!()` - I/Oエラーを作成
- `timeout_error!()` - タイムアウトエラーを作成
- `invalid_response_error!()` - 無効なレスポンスエラーを作成
- `prost_decode_error!()` - Protocol Buffers デコードエラーを作成

これらは非公開（`#[doc(hidden)]`）で、内部実装でのみ使用されます。

## 使用例

```rust
use tsubakuro_rust_core::prelude::*;

async fn example() -> Result<(), TgError> {
    // エラーが発生した場合
    let result: Result<(), TgError> = some_operation().await;
    
    match result {
        Err(TgError::ClientError(msg, cause)) => {
            eprintln!("クライアントエラー: {}", msg);
            if let Some(cause) = cause {
                eprintln!("原因: {}", cause);
            }
        }
        Err(TgError::TimeoutError(msg)) => {
            eprintln!("タイムアウト: {}", msg);
        }
        Err(TgError::IoError(msg, cause)) => {
            eprintln!("I/Oエラー: {}", msg);
            if let Some(cause) = cause {
                eprintln!("原因: {}", cause);
            }
        }
        Err(TgError::ServerError(_, msg, code, server_msg)) => {
            eprintln!("サーバーエラー: {}", msg);
            eprintln!("診断コード: {}", code);
            eprintln!("サーバーメッセージ: {}", server_msg);
        }
        Ok(_) => {}
    }
    
    result
}
```

## 関連モジュール

- [prelude.md](./prelude.md) - `TgError` の公開API

