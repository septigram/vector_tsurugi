# 開発

- [セットアップ](#セットアップ)
  - [DockerまたはPodman環境を使用する](#dockerまたはpodman環境を使用する)
  - [独自のツールボックスを使用する](#独自のツールボックスを使用する)
- [基本](#基本)
  - [ディレクトリ構造](#ディレクトリ構造)
  - [Makefile](#makefile)
  - [コードスタイル](#コードスタイル)
    - [ログスタイル](#ログスタイル)
    - [パニック](#パニック)
  - [機能フラグ](#機能フラグ)
  - [依存関係](#依存関係)
  - [最小サポートRustバージョン](#最小サポートrustバージョン)
- [ガイドライン](#ガイドライン)
  - [シンクヘルスチェック](#シンクヘルスチェック)
  - [内部ログレート制限の無効化](#内部ログレート制限の無効化)
- [テスト](#テスト)
  - [ユニットテスト](#ユニットテスト)
  - [統合テスト](#統合テスト)
  - [ブラックボックステスト](#ブラックボックステスト)
  - [プロパティテスト](#プロパティテスト)
  - [ヒントとコツ](#ヒントとコツ)
    - [`sccache`を使用した高速ビルド](#sccacheを使用した高速ビルド)
    - [特定のコンポーネントのテスト](#特定のコンポーネントのテスト)
    - [サンプルログの生成](#サンプルログの生成)
- [ベンチマーク](#ベンチマーク)
- [プロファイリング](#プロファイリング)
- [ドメイン](#ドメイン)
  - [Kubernetes](#kubernetes)
    - [アーキテクチャ](#アーキテクチャ)
      - [操作ロジック](#操作ロジック)
      - [どこで何を見つけるか](#どこで何を見つけるか)
    - [開発](#開発)
      - [要件](#要件)
      - [自動](#自動)
    - [テスト](#テスト-1)
      - [統合テスト](#統合テスト-1)
        - [要件](#要件-1)
        - [チュートリアル](#チュートリアル)

## セットアップ

Vectorでの作業に興味を持っていただき、大変嬉しく思います！始める前に、どのように開発したいかを選択してください。

小さな貢献や初めての貢献の場合は、Docker方式をお勧めします。自分でやりたいですか？それも問題ありません！

### DockerまたはPodman環境を使用する

> **ターゲット:** この方法を使用して、AARCH64、Arm6/7、およびx86/64 Linuxビルドを生成できます。

誰もが完全に動作するネイティブ環境を持っているわけではないため、私たちは環境をDocker（またはPodman）コンテナに詰め込みました！

これは「そのまま動作する」ことを望み、すぐに貢献を始めたいユーザーに最適です。また、CIでも使用しているため、壊れた場合は修正するまで他の作業ができません。😉

**次に進む前に、公式パッケージマネージャーまたは[Docker](https://docs.docker.com/get-docker/)または[Podman](https://podman.io/)のサイトからDockerまたはPodmanをインストールしてください。**

```bash
# オプション: `podman`を使用する場合のみ
export CONTAINER_TOOL="podman"
```

Linux環境でSELinuxがEnforcingモードで実行されている場合、vectorソースコードのチェックアウトを`container_home_t`コンテキストで再ラベルする必要があります。そうしないと、コンテナ環境がコードを読み書きできません：

```bash
cd your/checkout/of/vector/
sudo semanage fcontext -a "${PWD}(/.*)?" -t container_file_t
sudo restorecon . -R
```

デフォルトでは、`make environment`スタイルのタスクはGitHubのコンテナリポジトリから`docker pull`を実行します。朝のコーヒーを淹れながら、**オプションで**独自の環境をビルドできます☕：

```bash
# オプション: コーヒーを淹れに行きたい場合のみ
make environment-prepare
```

コーヒーができたら、シェルに入ることができます！

```bash
# 対話プロセス用に最適化されたマウントでシェルに入る。
# ここでは、完全なツールチェーンがあるかのようにVectorを使用できます（下記を参照！）
make environment
# 特定のコンテナツールを試す。（Docker/Podman）
make environment CONTAINER_TOOL="podman"
# 追加のCLIオプションを追加
make environment CLI_OPTS="--publish 3000:2000"
```

これで、以下の**「独自のツールボックスを使用する」**で詳しく説明されているジョブを使用できます。

環境の外から実行したいですか？_賢いですね。良い考えです。_ 以下のいずれかを実行できます：

```bash
# コードがコンパイルできることを確認
make check ENVIRONMENT=true
# コードが実際にコンパイルされることを確認（開発モード）
make build-dev ENVIRONMENT=true
# テストが合格することを確認
make test SCOPE="sources::example" ENVIRONMENT=true
# テスト（他のサービスを必要としない）が合格することを確認
make test ENVIRONMENT=true
# テストが合格することを確認（Dockerで必要なサービスを開始）
make test-integration SCOPE="sources::example" ENVIRONMENT=true
# ライブサービスに対してテストが合格することを確認。
make test-integration SCOPE="sources::example" AUTOSPAWN=false ENVIRONMENT=true
# すべてのテストが合格することを確認（Dockerで必要なサービスを開始）
make test-integration ENVIRONMENT=true
# ベンチマークを実行
make bench SCOPE="transforms::example" ENVIRONMENT=true
# プッシュする前にコードをフォーマット！
make fmt ENVIRONMENT=true
```

多くの貢献者がRustツールチェーンをローカルに保持することを選択しているため、明示的な環境オプトインを使用しています。

### 独自のツールボックスを使用する

> **ターゲット:** このオプションはMSVC/Mac/FreeBSDツールチェーンに必要です。任意の環境やOS用にビルドするために使用できます。

独自のホストでVectorをビルドするには、かなり完全な開発環境が必要です！

大まかに、以下が必要です：

- **Vectorをビルドするには:** 動作するRustup、Protobufツール、C++/Cビルドツール（LLVM、GCC、またはMSVC）、Python、Perl、`make`（できればGNU版）、`bash`、`cmake`、`GNU coreutils`、および`autotools`が必要です。
- **`make test`を実行するには:** [`cargo-nextest`](https://nexte.st/)をインストール
- **統合テストを実行するには:** `docker`を利用可能にするか、そのサービスの実際のライブバージョンを使用します。（`AUTOSPAWN=false`を使用）
- **`make check-component-features`を実行するには:** `remarshal`をインストール
- **`make check-licenses`または`make build-licenses`を実行するには:** `dd-rust-license-tool`を[インストール](https://github.com/DataDog/rust-license-tool)
- **`make generate-component-docs`を実行するには:** `cue`を[インストール](https://cuelang.org/docs/install/)

上記で説明したDocker環境内で何かを実行する必要がある場合、それは全く問題ありません。それらは衝突したり互いに害を与えたりしません。この場合、`make environment-generate`を実行するだけです。

シンプルなオプションが存在する場合、依存関係を減らすことに興味があります。アイデアがありますか？試してみてください。成功と失敗の両方を聞きたいです！

Vectorでの開発を行うには、主にいくつかのコマンドを使用します。`cargo`や`make`タスクなど、最も頻繁に実行されるものから順に使用できます：

```bash
# コードがコンパイルできることを確認
cargo check
make check
# コードが実際にコンパイルされることを確認（開発モード）
cargo build
make build-dev
# テストが合格することを確認
cargo test sources::example
make test SCOPE="sources::example"
# テスト（他のサービスを必要としない）が合格することを確認
cargo test
make test
# テストが合格することを確認（Dockerで必要なサービスを開始）
make test-integration SCOPE="sources::example"
# ライブサービスに対してテストが合格することを確認。
make test-integration SCOPE="sources::example" autospawn=false
cargo test --features docker sources::example
# すべてのテストが合格することを確認（Dockerで必要なサービスを開始）
make test-integration
# ベンチマークを実行
make bench SCOPE="transforms::example"
cargo bench transforms::example
# プッシュする前にコードをフォーマット！
make fmt
cargo fmt
# ウェブサイト用のコンポーネントドキュメントをビルド
make generate-component-docs
```

`make`を実行すると、すべてのタスクの完全なリストが表示されます。これらの一部はDockerコンテナを開始したり、コミットに署名したり、リリースを作成したりします。これらは一般的な開発コマンドではなく、結果は異なる場合があります。

## 基本

### ディレクトリ構造

- [`/.github`](../.github) - GitHubとCI関連の設定。
- [`/benches`](../benches) - 内部ベンチマーク。
- [`/config`](../config) - リリースに含まれる、公開向けのVector設定。
- [`/distribution`](../distribution) - さまざまなターゲット用の配布アーティファクト。
- [`/docs`](../docs) - Vector貢献者向けの内部ドキュメント。
- [`/lib`](../lib) - `vector`に依存しないが、プロジェクト内で使用される外部ライブラリ。
- [`/proto`](../proto) - Protobuf定義。
- [`/rfcs`](../rfcs) - 以前のVector提案。以前の決定に関するコンテキストを構築するのに最適な場所。
- [`/scripts`](../scripts) - ドキュメントを生成し、リポジトリを維持するために使用されるスクリプト。
- [`/src`](../src) - Vectorソース。
- [`/tests`](../tests) - さまざまな高レベルテストケース。
- [`/website`](../website) - VectorのウェブサイトとVectorユーザー向けの外部ドキュメント。

### Makefile

Vectorにはリポジトリのルートに[`Makefile`](../Makefile)が含まれています。これは
一般的なコマンドの高レベルインターフェースとして機能します。`make`を実行すると、
説明付きのmakeターゲットのリストが生成されます。これらのターゲットは
このドキュメント全体で参照されます。

### コードスタイル

コードのフォーマットには`stable`の`rustfmt`を使用し、CIはコードが
このフォーマットスタイルに従っていることを確認します。次のコマンドを実行するには、`rustfmt`が
ローカルのstableツールチェーンにインストールされていることを確認してください。

```bash
# rustfmtをインストールするには
rustup component add rustfmt

# コードをフォーマットするには
make fmt
```

#### ログスタイル

- ログイベントには常に[Tracing crate](https://tracing.rs/tracing/)のキー/値スタイルを使用します。
- イベントは大文字で始まり、ピリオド`.`で終わる必要があります。
- `e`や`err`は使用しないでください - 常に`error`と綴って、ログを豊かにし、
  出力が何であるかを明確にします。
- DebugよりもDisplayを優先し、`?error`ではなく`%error`を使用します。

ダメな例！

```rust
warn!("Failed to merge value: {}.", err);
```

良い例！

```rust
warn!(message = "Failed to merge value.", %error);
```

#### パニック

一般的なルールとして、Vectorのコードは*パニック*すべきではありません。

ただし、コードが特定の状態について特定の仮定を行い、それらの仮定が満たされない場合、これは明らかに
Vector内のバグによるものです。この状況では、Vectorは安全に続行できません。ここで
パニックを発行することは許容されます。

すべての潜在的なパニックは*関数ドキュメントに明確に文書化されている必要があります*。

### 機能フラグ

新しいコンポーネント（ソース、変換、またはシンク）が追加されると、対応する名前の
機能フラグの背後に配置する必要があります。これにより、Vectorビルドを
カスタマイズできるようになります。例については、`Cargo.toml`の`features`セクションを
参照してください。

さらに、特定のコンポーネントの開発中は、コンパイルを高速化するために
他のすべてのコンポーネントを無効にすると便利です。たとえば、
`console`シンクのみをビルドしてテストを実行できます：

```bash
cargo test --lib --no-default-features --features sinks-console sinks::console
```

テストが既にビルドされていて、コンポーネントファイルのみが変更された場合、
すべての機能でテストを再ビルドするよりも約4倍高速です。

### 依存関係

依存関係は*慎重に*選択し、可能であれば避けるべきです。依存関係のレビュー方法は
[レビューガイド](/docs/REVIEWING.md#dependencies)で確認できます。

依存関係が1つまたは複数のコンポーネントにのみ必要で、Vectorのコアには
必要ない場合、それをオプションにして、`Cargo.toml`のこれらのコンポーネントに対応する
機能の依存関係リストに追加します。

### 最小サポートRustバージョン

Vectorの最小サポートRustバージョン（MSRV）は、`Cargo.toml`で指定された`rust-version`で
示されます。

現在、VectorにはMSRVに関するポリシーはありません。依存関係によって必要になった場合、または
Vectorのコードベースで新しい言語機能を活用するために、いつでもバンプできます。

## ガイドライン

### シンクヘルスチェック

シンクは、環境と外部システムに対して設定を検証する手段としてヘルスチェックを実装できます。
理想的には、これによりシステムは、不十分な認証情報、到達不能な
エンドポイント、存在しないテーブルなどの問題をユーザーに通知できます。ただし、実行時に
発生する可能性のある問題を徹底的にチェックすることは不可能なため、完璧ではありません。

ヘルスチェックを実装する際は、偽陰性よりも偽陽性を優先します。
これは、シンクがその後失敗する場合でもヘルスチェックが合格することを好み、
シンクが正常に実行できた場合にヘルスチェックが失敗することを好まないことを意味します。

ヘルスチェックで偽陰性の一般的な原因は、シンク自体が
必要としない異なる失敗可能な操作を実行することです。たとえば、利用可能なすべてのS3
バケットをリストし、設定されたバケットがそのリストにあることを確認します。S3シンクは
すべてのバケットをリストする機能を必要とせず、それを知っているユーザーは
その権限を許可していない可能性があります。その場合、通常の操作には十分な認証情報があるにもかかわらず、
認証情報が悪いためにヘルスチェックが失敗します。

これは、シンク自体が行うことを模倣する一般的な戦略につながります。
残念ながら、ヘルスチェックが実際のイベントを利用できないという事実は、
ここでいくつかの制限につながります。これの最も明白な例は、
書き込みの正確なターゲットがイベント内の一部のフィールドの値に依存するシンク
（例：補間されたKinesisストリーム名）です。また、受信イベントが特定のスキーマに
準拠することが期待されるシンクでも発生します。どちらの場合も、ランダムなテストデータは
偽陰性の結果を引き起こす可能性がかなり高いです。より単純な場合でも、テストデータを
書き込むことの影響と、ユーザーがそれを驚きや侵入的と考えるかどうかを考える必要があります。
答えは通常、インターフェースしているシステムによって異なります。

上記のKinesisの例のように、場合によっては、何もしないことが正しいかもしれません。
扱っているエンティティ（この場合はKinesisストリーム）を把握するために動的情報が必要な場合、
それが正常に動作していることを意味のある方法で検証する方法を考え出す可能性は非常に低いです。
このようなデータ依存関係がある場合、何もしないことにフォールバックするヘルスチェックを
持つことは完全に有効です。

これらすべてを考慮して、新しいヘルスチェックを書く際に確認する簡単なチェックリストを以下に示します：

- [ ] このチェックは、シンク自体とは異なる失敗可能な操作を実行しますか？
- [ ] このチェックには、ユーザーが望ましくないと考える副作用（例：データ汚染）がありますか？
- [ ] このチェックが失敗するが、シンクが正常に動作する状況はありますか？

すべての答えが「いいえ」である必要はありませんが、「はい」が偽陰性につながる可能性と、
問題を見つけるためのチェック全体の有用性のバランスを考える必要があります。個別のヘルスチェックを
無効にするオプションがあるため、偽陰性の状況に陥ったユーザーには回避策があります。
私たちの目標は、ユーザーがそのレバーを引く必要がある可能性を最小限に抑えながら、
一般的な問題を検出するための良い努力をすることです。

### 内部ログレート制限の無効化

Vectorはデフォルトで独自の内部ログをレート制限します（10秒のウィンドウ）。開発中は、
すべてのログの発生を確認したい場合があります。

**グローバルに**（CLIフラグまたは環境変数）：

```bash
vector --config vector.yaml -r 1
# または
VECTOR_INTERNAL_LOG_RATE_LIMIT=1 vector --config vector.yaml
```

**ログステートメントごとに**：

```rust
// このログのレート制限を無効化
warn!(message = "Error occurred.", %error, internal_log_rate_limit = false);

// レート制限ウィンドウを1秒にオーバーライド
info!(message = "Processing batch.", batch_size, internal_log_rate_secs = 1);
```

## テスト

テストは非常に重要です。Vectorの主要な設計原則は信頼性であるためです。
Vectorのテスト方法の詳細については、
[テストブログ投稿](https://vector.dev/blog/how-we-test-vector/)をお読みください。

### ユニットテスト

ユニットテストは、Vectorのコード全体のインラインテストの大部分を指します。
ユニットテストの定義上の特徴は、外部サービスを必要としないため、
はるかに高速である必要があることです。次のコマンドで実行できます：

```bash
cargo test
```

### 統合テスト

統合テストは、Vectorが実際に統合しているサービスと
実際に動作することを確認します。ユニットテストとは異なり、統合テストは実行に
外部サービスが必要です。統合テストを設定する際のいくつかのルール：

- [ ] すべての貢献者が統合テストを実行できるようにするため、サービスは
      Dockerコンテナで実行する必要があります。
- [ ] サービスは、環境変数を通じて設定された一意のポートで
      設定する必要があります。
- [ ] Vectorの[`Makefile`](/Makefile)に`test-integration-<name>`を追加し、
      統合テストを実行する前にサービスが開始されることを確認します。
- [ ] 統合の名前を、Vectorの
      [`.github/workflows/integration-test.yml`](../.github/workflows/integration-test.yml)ワークフローの
      `test-integration`ジョブのincludeマトリックスに追加します。

完了したら、次のコマンドで統合テストを実行できます：

```bash
make test-integration-<name>
```

### ブラックボックステスト

Vectorは、[Vectorのテストハーネス](https://github.com/vectordotdev/vector-test-harness)を介して
ブラックボックステストも提供します。これは
実際の環境でVectorのパフォーマンスをテストする複雑なテストスイートです。
通常、ベンチマークに使用されますが、正確性テストにも使用されます。

これらのテストは、[CIセクション](CONTRIBUTING.md)で説明されているように、PR内で実行できます。

### プロパティテスト

Vectorは、プロパティテストには[Proptest](https://github.com/proptest-rs/proptest)の使用を推奨します。

### ヒントとコツ

#### `sccache`を使用した高速ビルド

Vectorは、多くの依存関係を持つ大規模なプロジェクトです。別のブランチに変更したり、
`cargo clean`を実行したりすると、多くの依存関係を再ビルドする必要がある場合があり、
生産性に影響を与えます。このサイクル時間の一部を減らす方法の1つは、`sccache`を使用することです。
これは、コンパイルアセットをキャッシュして、何度も再コンパイルすることを避けます。

`sccache`は、`rustc`の前に配置されるように設定され、Cargoからコンパイルリクエストを受け取り、
キャッシュされたコンパイルユニットが既にキャッシュにあるかどうかを確認することで機能します。
キャッシュされたアセットを使用する前に、異なるコンパイラフラグ、Rustのバージョンなどが
考慮されることを確認します。

`sccache`を使用するには、まず[インストール](https://github.com/mozilla/sccache#installation)
する必要があります。主要なプラットフォーム用のプリビルドバイナリがあり、すぐに始められます。
[使用方法](https://github.com/mozilla/sccache#usage)ドキュメントでは、実際に使用するために
環境を設定する方法も説明しています。`$HOME/.cargo/config`アプローチを使用することをお勧めします。
これは、Vectorの開発だけでなく、すべてのRust開発作業を高速化するのに役立ちます。

`sccache`は元々、CIワーカー間で再利用性を最大化するために、クラウドストレージにコンパイルアセットを
キャッシュするように設計されましたが、`sccache`は実際にはデフォルトでローカルにアセットを
保存することをサポートしています。ローカルモードはローカル開発に適しています。
キャッシュされたアセットに問題が発生した場合、キャッシュディレクトリを削除するのがはるかに簡単です。
また、追加のインフラストラクチャや費用もかかりません。

#### 特定のコンポーネントのテスト

特定のコンポーネントを開発していて、このコンポーネントに関連するユニットテストのみを
迅速に反復したい場合、次のアプローチで待機時間を短縮できます：

1. [cargo-watch](https://github.com/passcod/cargo-watch)をインストールします。
2. （GNU/Linuxのみ）LLVM 9をインストール（たとえば、Debianのパッケージ`llvm-9`）
   し、`RUSTFLAGS`環境変数を設定して`lld`をリンカーとして使用します：

   ```sh
   export RUSTFLAGS='-Clinker=clang-9 -Clink-arg=-fuse-ld=lld'
   ```

3. Vectorのソースのルートディレクトリで実行

   ```sh
   cargo watch -s clear -s \
     'cargo test --lib --no-default-features --features=<component type>-<component id> <component type>::<component id>'
   ```

   たとえば、コンポーネントが`reduce`変換の場合、上記のコマンドは
   次のようになります：

   ```sh
   cargo watch -s clear -s \
     'cargo test --lib --no-default-features --features=transforms-reduce transforms::reduce'
   ```

#### サンプルログの生成

ファイルからログを送信するテスト用のサンプルログファイルセットを構築するために`flog`を使用します。
これは、`homebrew`を使用するMacで次のコマンドで実行できます。
flogのインストール手順は
[こちら](https://github.com/mingrammer/flog#installation)で見つけることができます。

```bash
flog --bytes $((100 * 1024 * 1024)) > sample.log
```

これにより、`sample.log`ファイルに`100MiB`のサンプルログファイルが作成されます。

## ベンチマーク

すべてのベンチマークは[`/benches`](/benches)フォルダに配置されます。
`make bench`コマンドでベンチマークを実行できます。さらに、Vectorは
複雑なエンドツーエンドの統合とパフォーマンステスト用の完全な[テストハーネス](https://github.com/vectordotdev/vector-test-harness)
を維持しています。

## プロファイリング

Vectorのパフォーマンスを改善しようとしている（または変更が悪化した理由を理解しようとしている）場合、
プロファイリングは時間がどこで費やされているかを確認するのに役立つツールです。

有用なプロファイリングツールはたくさんありますが、簡単に始められる場所は
Linuxの`perf`です。始める前に、統計を収集するためのアクセス権を
自分に与える必要があるでしょう：

```sh
echo -1 | sudo tee /proc/sys/kernel/perf_event_paranoid
```

また、`Cargo.toml`を編集して、Vectorがリリースモードでデバッグシンボル付きで
ビルドされていることを確認する必要があります。これにより、最終的な出力で
人間が読める情報が得られます：

```toml
[profile.release]
debug = true
```

次に、プロファイリングに興味のある設定でVectorのリリースビルドを
起動できます。

```sh
cargo run --release -- --config my_test_config.toml
```

起動したら、`ps`ツール（または同等のもの）を使用して、そのPIDをメモします。
これを使用して、`perf`にデータを収集したいプロセスを伝えます。

次のステップは、テストしている設定に多少依存します。この
例では、ポート9000でリッスンしているシンプルなTCPモードのソケットソースを
使用していると仮定します。また、`access.log`に大きなサンプル入力ファイルが
あると仮定します（`flog`のようなツールを使用してこれを生成できます）。

これらすべてが準備できたら、テスト入力をVectorに送信し、負荷下で
データを収集できます：

```sh
perf record -F99 --call-graph dwarf -p $VECTOR_PID socat -dd OPEN:access.log TCP:localhost:9000
```

これは、`socat`コマンドの実行中、既に実行中のVectorプロセスから
`perf`にデータを収集するように指示します。`-F`引数は、
`perf`がVectorのコールスタックをサンプリングする頻度です。より高い頻度は
より多くのデータを収集し、より詳細な出力を生成しますが、処理に非常に長い時間がかかる
膨大な量のデータを生成する可能性があります。入力データが処理に1分以上かかるほど
大きい場合、`-F99`はうまく機能しますが、設定に応じて入力サイズとサンプリング頻度の両方を
調整してください。

これは、`perf`でプログラムをプロファイリングする通常の方法ではないことに注意してください。
通常、`perf record my_program`のようなものを単純に実行し、
PIDなどを心配する必要はありません。これとは異なるのは、Vectorが負荷下で
何をしているかについてのデータのみに関心があるためです。`perf`の下で直接実行すると、
プロセスの全期間（起動、シャットダウン、アイドル時間を含む）のデータが収集されます。
負荷生成コマンドの実行中のみ`perf`にデータを収集するように指示することで、
より焦点を絞ったデータセットが得られ、異なるコマンドを連続してタイミングを合わせることを
心配する必要がありません。

現在のディレクトリに、収集されたすべての情報を含む`perf.data`ファイルが見つかります。
これを処理する方法はいくつかありますが、最も有用な方法の1つは
[flamegraph](http://www.brendangregg.com/flamegraphs.html)を作成することです。このために
`inferno`ツール（`cargo install`で利用可能）を使用できます：

```sh
perf script | inferno-collapse-perf > stacks.folded
cat stacks.folded | inferno-flamegraph > flamegraph.svg
```

これで完了です！お気に入りのWebブラウザで開いてナビゲートできるflamegraph SVGファイルが
できました。

## ドメイン

このセクションには、Vectorのさまざまな領域に関するドメイン固有の開発知識が含まれています。
開発領域に関連するドメインについて、このセクションをスキャンする必要があります。

### Kubernetes

#### アーキテクチャ

Kubernetes統合アーキテクチャは、主に
[RFC 2221](../rfcs/2020-04-04-2221-kubernetes-integration.md)に触発されているため、これは
概念の深い説明ではなく、効果的な設計の簡潔な概要です。

##### 操作ロジック

`kubernetes_logs`ソースを使用すると、VectorはKubernetes APIに接続し、
Vector自体が実行されている同じ`Node`で実行されている`Pod`に対して
ストリーミングウォッチリクエストを実行します。Vectorが`Node`で実行されている
すべての`Pod`のリストを取得すると、各`Pod`に対応するログファイルの
ログの収集を開始します。プレーンテキスト（非gzip圧縮）ファイルのみが
考慮されます。
ログファイルはイベントに解析され、これらのイベントは
対応する`Pod`からのメタデータで注釈付けされ、元のログファイルの
ファイルパスを介して相関付けられます。
イベントはその後、トポロジに渡されます。

##### どこで何を見つけるか

カスタムKubernetes APIクライアントとマシンリーを使用しています。
これは`src/kubernetes`にあります。
`kubernetes_logs`ソースは`src/sources/kubernetes_logs`にあります。
エンドツーエンド（E2E）テストフレームワークもあり、
これは`lib/k8s-test-framework`にあり、そのフレームワークを使用する
実際のエンドツーエンドテストは`lib/k8s-e2e-tests`にあります。

`distribution/docker`、`distribution/kubernetes`にあるKubernetes関連の配布ビットと
Helmチャートは、[`vectordotdev/helm-charts`](https://github.com/vectordotdev/helm-charts/)で
見つけることができます。

開発支援リソースは`Tiltfile`と`tilt`ディレクトリにあります。

#### 開発

`kubernetes_logs`ソースや`deployment/kubernetes/*.yaml`設定など、
Kubernetesで動作するように設計されたVectorの一部を開発する場合、
特別なフローがあります。

このフローは、Vectorをビルドしてクラスターにデプロイすることを容易にします。

##### 要件

Vectorで作業するために通常必要なものに加えて、いくつかの追加要件があります：

- [`tilt`](https://tilt.dev/)
- [`docker`](https://www.docker.com/)
- [`kubectl`](https://kubernetes.io/docs/tasks/tools/install-kubectl/)
- [`minikube`](https://minikube.sigs.k8s.io/)で動作する、またはその他のk8sクラスター

##### 自動

`tilt`を使用して変更を検出し、イメージを再ビルドし、Kubernetesリソースを
更新できます。ローカルKubernetesクラスターを起動し、Vectorのルートディレクトリから
`tilt up`を実行するだけです。

#### テスト

##### 統合テスト

Kubernetes統合テストには、問題が発生する可能性のある多くの部分があります。

複雑さに対処し、高品質を維持するために、E2E（エンドツーエンド）テストを
使用しています。

> E2Eテストは通常CIで実行されるため、通常は手動で実行する必要はありません。

###### 要件

- `kubernetes`クラスター（`minikube`には特別なサポートがありますが、任意のクラスターが
  動作するはずです）
- `docker`
- `kubectl`
- `bash`
- `cross` - `cargo install cross`
- [`helm`](https://helm.sh/)

VectorリリースアーティファクトはE2Eテスト用に準備されているため、それを行う能力も
必要です。詳細については、Vector [docs](https://vector.dev)を参照してください。

注意：

> - `minikube`には、テストプロセスに影響を与えた`1.12.x`バージョンにバグがありました
>   - <https://github.com/kubernetes/minikube/issues/8799>を参照してください。
>   このバグが修正されたバージョン`1.13.0+`を使用してください。
> - `minikube`はZFSシステムで実行する際に問題があります。ZFSを使用している場合、
>   クラウドクラスターまたはローカルレジストリ付きの[`minik8s`](https://microk8s.io/)の使用を
>   お勧めします。
> - E2Eテストは、完全なVectorビルドを実行するのに十分なリソースがあることを期待しています。
>   通常、2CPUで8GBのRAMがあれば、ローカルでE2Eテストを正常に完了するのに十分です。

###### チュートリアル

E2Eテストを実行するには、次のコマンドを使用します：

```shell
CONTAINER_IMAGE_REPO=<your name>/vector-test make test-e2e-kubernetes
```

ここで、`CONTAINER_IMAGE_REPO`は使用するdockerイメージリポジトリ名で、`:`の後の部分は
含みません。`<your name>`をDocker Hubのユーザー名に置き換えます。

テストの動作を調整するために、追加のパラメータを渡すこともできます：

- `QUICK_BUILD=true` - 開発ビルドとdevフローからのイメージを
  プロダクションdockerイメージの代わりに使用します。準備プロセスを大幅に高速化しますが、
  リリースビルドでの正確性を保証しません。テストやVectorコードの開発で
  反復サイクルを高速化するのに役立ちます。

- `USE_MINIKUBE_CACHE=true` - ビルドされたdockerイメージを指定された名前で
  レジストリにプッシュする代わりに、`minikube`制御のクラスターノードに
  直接イメージをロードします。
  `minikube`クラスターに対してテストする必要があります。テストを実行するために
  レジストリが不要になります。
  `USE_MINIKUBE_CACHE=true`が設定されている場合、`CONTAINER_IMAGE_REPO`のデフォルト値を
  提供するため、省略できます。
  `auto`（デフォルト）に設定して、現在の`kubectl`コンテキストに基づいて
  `minikube cache`を使用するかどうかを自動検出できます。オプトアウトするには、
  `USE_MINIKUBE_CACHE=false`を設定します。

- `CONTAINER_IMAGE=<your name>/vector-test:tag` - Vector dockerイメージを
  ビルドするステップを完全にスキップし、代わりに指定されたイメージを使用します。
  既にテストしたいVector dockerイメージがある場合、反復速度を高速化するのに
  役立ちます。

- `SKIP_CONTAINER_IMAGE_PUBLISHING=true` - イメージ公開ステップを完全にスキップします。
  反復速度を高速化したい場合、およびテストしているクラスターに対して
  テストしたいVectorイメージが既に利用可能であることがわかっている場合に役立ちます。

- `SCOPE` - `cargo test`コマンドにフィルタを渡してテストをフィルタリングします。
  実質的に`cargo test -- $SCOPE`と同等です。

追加のコマンドを渡すには、次のようにします：

```shell
QUICK_BUILD=true USE_MINIKUBE_CACHE=true make test-e2e-kubernetes
```

または

```shell
QUICK_BUILD=true CONTAINER_IMAGE_REPO=<your name>/vector-test make test-e2e-kubernetes
```

