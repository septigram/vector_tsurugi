# job モジュール仕様

## 概要

`job` モジュールは、非同期処理の結果を取得するための `Job` 型と、ジョブのキャンセルを管理する `CancelJob` 型を提供します。

このモジュールは非公開（`#[doc(hidden)]`）ですが、`prelude` モジュールを通じて `Job` と `CancelJob` が公開されています。

## 主要な型

### Job<T>

非同期処理の結果を表すジェネリック型です。レスポンスデータを非同期で提供し、完了前にキャンセルすることができます。

**スレッド安全性**: スレッド非安全（thread unsafe）

```rust
pub struct Job<T> {
    // 内部フィールドは非公開
}
```

#### メソッド

##### name()

ジョブ名を取得します。

```rust
pub fn name(&self) -> &String
```

##### set_default_timeout()

デフォルトタイムアウトを設定します。

```rust
pub fn set_default_timeout(&mut self, timeout: Duration)
```

##### wait()

レスポンスの到着を待機します。

```rust
pub async fn wait(&mut self, timeout: Duration) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - レスポンスを受信
- `Ok(false)` - タイムアウト

##### is_done()

レスポンスが到着しているかどうかを確認します。

```rust
pub async fn is_done(&mut self) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - レスポンスを受信
- `Ok(false)` - レスポンス未受信

##### take()

結果値を取得します。レスポンスが到着するまで待機します。

結果値は一度のみ取得できます。2回目の呼び出しはエラーを返します。

```rust
pub async fn take(&mut self) -> Result<T, TgError>
```

デフォルトタイムアウトを使用します。

##### take_for()

結果値を取得します。指定されたタイムアウトでレスポンスが到着するまで待機します。

```rust
pub async fn take_for(&mut self, timeout: Duration) -> Result<T, TgError>
```

##### take_if_ready()

レスポンスが到着している場合のみ結果値を取得します。

```rust
pub async fn take_if_ready(&mut self) -> Result<Option<T>, TgError>
```

**戻り値**:
- `Ok(Some(value))` - 結果値
- `Ok(None)` - レスポンス未受信

##### cancel()

ジョブをキャンセルし、キャンセル完了を待機します。

```rust
pub async fn cancel(self) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - レスポンスは既に受信済み、またはキャンセルが既に開始済み
- `Ok(false)` - タイムアウト

**注意**: レスポンスは必ずしも `OPERATION_CANCELED` ではありません。タイミングによっては通常の処理結果が返される場合があります。

##### cancel_for()

ジョブをキャンセルし、指定されたタイムアウトでキャンセル完了を待機します。

```rust
pub async fn cancel_for(self, timeout: Duration) -> Result<bool, TgError>
```

##### cancel_async()

ジョブのキャンセルを開始します。

```rust
pub async fn cancel_async(mut self) -> Result<Option<CancelJob>, TgError>
```

**戻り値**:
- `Ok(Some(CancelJob))` - キャンセルを開始
- `Ok(None)` - キャンセルできなかった（レスポンス既に受信済み、またはキャンセル既に開始済み）

##### close()

リソースを破棄します。

レスポンスが未受信でキャンセルも未実行の場合、キャンセルを実行します（レスポンスは待機しません）。

```rust
pub async fn close(mut self) -> Result<(), TgError>
```

#### Drop トレイトの実装

`Job` がドロップされる際、自動的にキャンセル要求が送信されます。ただし、`done`、`canceled`、`closed` が `true` の場合、または既にレスポンスが存在する場合は何も行いません。

### CancelJob

ジョブのキャンセル処理の完了を待機するための型です。

**スレッド安全性**: スレッド非安全（thread unsafe）

```rust
pub struct CancelJob {
    // 内部フィールドは非公開
}
```

#### メソッド

##### wait()

レスポンスの到着を待機します。

```rust
pub async fn wait(&mut self, timeout: Duration) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - レスポンスを受信
- `Ok(false)` - タイムアウト

##### is_done()

レスポンスが到着しているかどうかを確認します。

```rust
pub async fn is_done(&mut self) -> Result<bool, TgError>
```

**戻り値**:
- `Ok(true)` - レスポンスを受信
- `Ok(false)` - レスポンス未受信

## 使用例

### 基本的な使用

```rust
use tsubakuro_rust_core::prelude::*;
use std::time::Duration;

async fn example(client: &SqlClient, transaction: &Transaction) -> Result<(), TgError> {
    let sql = "insert into tb values(1, 'abc')";
    let mut job = client.execute_async(transaction, sql).await?;

    // 結果を待機して取得
    let execute_result = job.take_for(Duration::from_secs(10)).await?;
    println!("inserted rows={}", execute_result.inserted_rows());

    Ok(())
}
```

### 非ブロッキング処理

```rust
async fn example(mut job: Job<SqlExecuteResult>) -> Result<(), TgError> {
    loop {
        let done = job.is_done().await?;
        if done {
            let execute_result = job.take().await?;
            break;
        }
        // 他の処理を実行
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}
```

### キャンセルの使用

```rust
async fn example(mut job: Job<SqlExecuteResult>) -> Result<(), TgError> {
    // キャンセルを開始
    if let Some(mut cancel_job) = job.cancel_async().await? {
        // キャンセル完了を待機
        cancel_job.wait(Duration::from_secs(10)).await?;
    }

    Ok(())
}
```

### タイムアウト付き待機

```rust
async fn example(mut job: Job<SqlExecuteResult>) -> Result<(), TgError> {
    let done = job.wait(Duration::from_secs(5)).await?;
    if done {
        let execute_result = job.take().await?;
    } else {
        eprintln!("タイムアウトしました");
    }

    Ok(())
}
```

## 注意事項

1. **スレッド安全性**: `Job` と `CancelJob` はスレッド非安全です。複数のスレッドから同時にアクセスしないでください。

2. **take() の呼び出し**: `take()` は一度のみ呼び出すことができます。2回目の呼び出しはエラーを返します。

3. **リソース管理**: `Job` をドロップする前に `close()` を明示的に呼び出すことを推奨しますが、ドロップ時にも自動的にキャンセル要求が送信されます。

4. **キャンセルのタイミング**: `cancel()` を呼び出しても、必ずしも `OPERATION_CANCELED` レスポンスが返されるとは限りません。タイミングによっては通常の処理結果が返される場合があります。

## 関連モジュール

- [prelude.md](./prelude.md) - `Job` と `CancelJob` の公開API
- [service.md](./service.md) - 非同期APIの使用方法

