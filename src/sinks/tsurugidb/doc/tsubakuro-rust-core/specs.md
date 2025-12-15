# tsubakuro-rust-core 仕様書

## 概要

`tsubakuro-rust-core` は、RustでTsurugiデータベースにアクセスするためのコアライブラリです。

このライブラリは [Tsubakuro/Java](https://github.com/project-tsurugi/tsubakuro) から移植されたもので、Tsurugi DBサーバーとの通信とSQL操作を提供します。

## バージョン要件

- **Tsurugi**: 1.7.0 以降
- **Rust**: 1.84.1 以降 (MSRV)

## 制限事項

- SQLサービスのみ提供
- TCP接続のみ利用可能

## クレート機能

デフォルト機能には以下が含まれます：

- `with_bigdecimal` - [bigdecimal](https://crates.io/crates/bigdecimal) によるdecimalサポートを有効化
- `with_rust_decimal` - [rust_decimal](https://crates.io/crates/rust_decimal) によるdecimalサポートを有効化
- `with_chrono` - [chrono](https://crates.io/crates/chrono) による日時サポートを有効化
- `with_time` - [time](https://crates.io/crates/time) による日時サポートを有効化

## モジュール構成

このライブラリは以下のモジュールで構成されています：

### パブリックモジュール

- **prelude** - 主要な型とトレイトをエクスポートするモジュール
  - 詳細: [prelude.md](./prelude.md)

### 非公開モジュール（内部実装）

- **error** - エラー処理
  - 詳細: [error.md](./error.md)
- **job** - 非同期ジョブ管理
  - 詳細: [job.md](./job.md)
- **service** - サービスクライアント
  - 詳細: [service.md](./service.md)
- **session** - セッション管理
  - 詳細: [session.md](./session.md)
- **transaction** - トランザクション管理
  - 詳細: [transaction.md](./transaction.md)
- **util** - ユーティリティ機能
  - 詳細: [util.md](./util.md)

## 基本的な使用フロー

SQLを実行するための一般的な手順は以下の通りです：

1. **セッションの作成**（Tsurugi DBサーバーへの接続）
   - `ConnectionOption` を作成
   - エンドポイントURL（例：`tcp://localhost:12345`）を設定
   - `Credential` を設定
   - `Session::connect()` を呼び出し

2. **SqlClient の作成**
   - `Session::make_client()` を呼び出し

3. **プリペアドステートメントの作成**（使用する場合）
   - `SqlClient::prepare()` を呼び出し

4. **トランザクションの開始**（`Transaction` の作成）
   - `TransactionOption` を作成
   - `TransactionType` などを設定
   - `SqlClient::start_transaction()` を呼び出し

5. **SQLの実行**
   - `SqlClient::execute()` または `prepared_execute()`
   - `SqlClient::query()` または `prepared_query()`

6. **トランザクションのコミット**
   - `CommitOption` を作成
   - `CommitType` などを設定
   - `SqlClient::commit()` を呼び出し

7. **トランザクションのクローズ**
   - `Transaction::close()` を呼び出し

8. **プリペアドステートメントのクローズ**（作成した場合）
   - `SqlPreparedStatement::close()` を呼び出し

9. **セッションのクローズ**
   - `Session::close()` を呼び出し

## 使用例

### 基本的な接続とSQL実行

```rust
use std::time::Duration;
use tsubakuro_rust_core::prelude::*;

async fn example() -> Result<(), TgError> {
    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url("tcp://localhost:12345")?;
    connection_option.set_application_name("Tsubakuro/Rust example");
    connection_option.set_session_label("example session");
    connection_option.set_default_timeout(Duration::from_secs(10));

    let session = Session::connect(&connection_option).await?;
    let client: SqlClient = session.make_client();

    // トランザクションの開始
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::Short);
    let transaction = client.start_transaction(&transaction_option).await?;

    // SQLの実行
    let sql = "insert into customer values(1, 'Alice', 25)";
    let execute_result = client.execute(&transaction, sql).await?;
    println!("inserted rows={}", execute_result.inserted_rows());

    // コミット
    let commit_option = CommitOption::default();
    client.commit(&transaction, &commit_option).await?;

    // リソースのクリーンアップ
    transaction.close().await?;
    session.close().await?;

    Ok(())
}
```

## 依存関係

このライブラリは以下の主要な依存関係を使用しています：

- `tokio` - 非同期ランタイム
- `prost` - Protocol Buffers サポート
- `url` - URL解析
- `chrono` - 日時処理（オプション）
- `bigdecimal` - 高精度数値演算（オプション）
- `rust_decimal` - 10進数演算（オプション）

## 関連ドキュメント

各モジュールの詳細な仕様については、以下のドキュメントを参照してください：

- [error.md](./error.md) - エラー処理の仕様
- [job.md](./job.md) - ジョブ管理の仕様
- [prelude.md](./prelude.md) - 公開APIの仕様
- [service.md](./service.md) - サービスクライアントの仕様
- [session.md](./session.md) - セッション管理の仕様
- [transaction.md](./transaction.md) - トランザクション管理の仕様
- [util.md](./util.md) - ユーティリティ機能の仕様

## ライセンス

[Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

