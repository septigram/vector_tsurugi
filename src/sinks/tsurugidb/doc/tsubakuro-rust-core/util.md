# util モジュール仕様

## 概要

`util` モジュールは、tsubakuro-rust-core の内部実装で使用されるユーティリティ機能を提供します。

このモジュールは非公開（`pub(crate)`）で、内部実装でのみ使用されます。

## 主要な型と関数

### Timeout

タイムアウト管理を行う構造体です。

```rust
pub(crate) struct Timeout {
    timeout: Duration,
    start: Instant,
}
```

#### メソッド

##### new()

新しい `Timeout` インスタンスを作成します。

```rust
pub(crate) fn new(timeout: Duration) -> Timeout
```

##### is_timeout()

タイムアウトが発生しているかどうかを確認します。

```rust
pub(crate) fn is_timeout(&self) -> bool
```

**戻り値**:
- `true` - タイムアウトが発生
- `false` - タイムアウト未発生

**注意**: `timeout` が `Duration::ZERO` の場合、常に `false` を返します。

### string_to_prost_string()

`Option<&String>` を `prost::alloc::string::String` に変換するヘルパー関数です。

```rust
pub(crate) fn string_to_prost_string(s: Option<&String>) -> ProstString
```

**戻り値**:
- `Some(s)` の場合: `s` の内容を持つ `ProstString`
- `None` の場合: 空の `ProstString`

## 内部マクロ

### return_err_if_timeout!

タイムアウトが発生した場合にエラーを返すマクロです。

```rust
#[macro_export]
macro_rules! return_err_if_timeout {
    ($timeout:expr, $function_name:expr) => {
        if $timeout.is_timeout() {
            ::log::trace!("{}: timeout", $function_name);
            return Err($crate::timeout_error!($function_name))
        }
    };
}
```

**使用方法**:
```rust
return_err_if_timeout!(timeout, "function_name");
```

このマクロは、タイムアウトが発生した場合にトレースログを出力し、`TimeoutError` を返します。

## 使用例

### Timeout の使用

```rust
use std::time::Duration;
use tsubakuro_rust_core::util::Timeout;

// タイムアウトを10秒に設定
let timeout = Timeout::new(Duration::from_secs(10));

// 処理を実行
loop {
    return_err_if_timeout!(timeout, "my_function");
    
    // 何らかの処理
    // ...
    
    // タイムアウトチェック
    if timeout.is_timeout() {
        break;
    }
}
```

### string_to_prost_string の使用

```rust
use tsubakuro_rust_core::util::string_to_prost_string;

let some_string = Some(&String::from("hello"));
let prost_string = string_to_prost_string(some_string);
// prost_string は "hello" を含む

let none_string: Option<&String> = None;
let empty_prost_string = string_to_prost_string(none_string);
// empty_prost_string は空文字列
```

## 実装の詳細

### Timeout

`Timeout` は、開始時刻（`Instant`）とタイムアウト期間（`Duration`）を保持し、経過時間を計算してタイムアウトかどうかを判断します。

- タイムアウト期間が `Duration::ZERO` の場合、タイムアウトは発生しません（無限待機を意味します）
- `is_timeout()` は、現在時刻から開始時刻までの経過時間がタイムアウト期間を超えているかどうかを確認します

### string_to_prost_string

この関数は、`prost` クレートで使用される `String` 型に変換するためのヘルパー関数です。`Option` 型を扱う際の利便性のために提供されています。

## テスト

このモジュールには、`Timeout` の動作を確認するためのテストが含まれています：

- ゼロタイムアウトは常に `false` を返す
- 指定された期間後にタイムアウトが発生する

## 注意事項

1. **非公開モジュール**: このモジュールは内部実装用であり、公開APIの一部ではありません。

2. **タイムアウトの精度**: `Timeout` は `tokio::time::Instant` を使用しているため、システムクロックの精度に依存します。

3. **ゼロタイムアウト**: `Duration::ZERO` はタイムアウトしないことを意味しますが、実際の実装では無限待機を表すために使用されます。

## 関連モジュール

- [error.md](./error.md) - `TimeoutError` の定義
- [session.md](./session.md) - タイムアウトを使用するセッション処理

