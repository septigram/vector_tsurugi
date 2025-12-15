use futures::FutureExt;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tsubakuro_rust_core::prelude::*;

use vector_lib::{
    config::AcknowledgementsConfig,
    configurable::{component::GenerateConfig, configurable_component},
    sink::VectorSink,
};

use super::{
    service::{TsurugiRetryLogic, TsurugiService},
    sink::TsurugiSink,
};
use crate::{
    config::{Input, SinkConfig, SinkContext},
    sinks::{
        Healthcheck,
        util::{
            BatchConfig, RealtimeSizeBasedDefaultBatchSettings, ServiceBuilderExt,
            TowerRequestConfig,
        },
    },
};

const fn default_pool_size() -> u32 {
    5
}

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

/// Configuration for the `tsurugidb` sink.
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

impl GenerateConfig for TsurugiConfig {
    fn generate_config() -> toml::Value {
        toml::from_str(
            r#"endpoint = "tcp://localhost:12345"
            table = "table"
        "#,
        )
        .unwrap()
    }
}

#[async_trait::async_trait]
#[typetag::serde(name = "tsurugidb")]
impl SinkConfig for TsurugiConfig {
    async fn build(&self, _cx: SinkContext) -> crate::Result<(VectorSink, Healthcheck)> {
        // 1. ConnectionOptionの作成
        let mut connection_option = ConnectionOption::new();
        connection_option.set_endpoint_url(&self.endpoint)
            .map_err(|e| crate::Error::from(format!("Failed to set endpoint URL: {}", e)))?;
        connection_option.set_application_name("Vector Tsurugi Sink");
        connection_option.set_default_timeout(Duration::from_secs(10));

        // 2. セッションの作成（接続プールの代わり）
        // 注意: 現在のtsubakuro-rust-coreは接続プールを提供していないため、
        // セッションを直接保持する
        let session = Session::connect(&connection_option).await
            .map_err(|e| crate::Error::from(format!("Failed to connect to Tsurugi: {}", e)))?;

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

async fn healthcheck(session: Arc<Session>) -> crate::Result<()> {
    let client: SqlClient = session.make_client();
    
    // 簡単なSELECT文で接続確認
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::ReadOnly);
    
    let transaction = client.start_transaction(&transaction_option).await
        .map_err(|e| crate::Error::from(format!("Failed to start transaction: {}", e)))?;
    
    // TsurugiDBではSELECT 1がサポートされていないため、
    // トランザクションが正常に開始できれば接続は成功とみなす
    // トランザクションをクローズして正常終了
    transaction.close().await
        .map_err(|e| crate::Error::from(format!("Failed to close transaction: {}", e)))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_config() {
        crate::test_util::test_generate_config::<TsurugiConfig>();
    }

    #[test]
    fn parse_config() {
        let cfg = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://localhost:12345"
            table = "mytable"
        "#,
        )
        .unwrap();
        assert_eq!(cfg.endpoint, "tcp://localhost:12345");
        assert_eq!(cfg.table, "mytable");
        // デフォルト値の確認
        assert_eq!(cfg.pool_size, 5);
        assert_eq!(cfg.transaction_type, "occ");
    }

    #[test]
    fn parse_config_with_custom_values() {
        let cfg = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://example.com:54321"
            table = "test_table"
            pool_size = 10
            transaction_type = "ltx"
        "#,
        )
        .unwrap();
        assert_eq!(cfg.endpoint, "tcp://example.com:54321");
        assert_eq!(cfg.table, "test_table");
        assert_eq!(cfg.pool_size, 10);
        assert_eq!(cfg.transaction_type, "ltx");
    }

    #[test]
    fn parse_config_with_default_transaction_type() {
        // transaction_typeが指定されていない場合、デフォルトで"occ"が使用される
        let cfg = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://localhost:12345"
            table = "mytable"
        "#,
        )
        .unwrap();
        assert_eq!(cfg.transaction_type, "occ");
    }

    #[test]
    fn parse_config_invalid_transaction_type() {
        // 不正なtransaction_typeの場合、パースは成功するがbuild時にエラーになる
        // パース段階では文字列として受け取るため、ここではパースが成功することを確認
        let cfg = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://localhost:12345"
            table = "mytable"
            transaction_type = "invalid"
        "#,
        )
        .unwrap();
        assert_eq!(cfg.transaction_type, "invalid");
        // parse_transaction_type関数が正しくエラーを返すことを確認
        assert!(parse_transaction_type("invalid").is_err());
        assert!(parse_transaction_type("occ").is_ok());
        assert!(parse_transaction_type("ltx").is_ok());
        assert!(parse_transaction_type("OCC").is_ok()); // 大文字小文字を区別しない
        assert!(parse_transaction_type("LTX").is_ok());
    }

    #[test]
    fn parse_config_missing_required_fields() {
        // endpointが欠けている場合
        let result = toml::from_str::<TsurugiConfig>(
            r#"
            table = "mytable"
        "#,
        );
        assert!(result.is_err());

        // tableが欠けている場合
        let result = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://localhost:12345"
        "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn parse_config_unknown_fields() {
        // serde(deny_unknown_fields)により、未知のフィールドは拒否される
        let result = toml::from_str::<TsurugiConfig>(
            r#"
            endpoint = "tcp://localhost:12345"
            table = "mytable"
            unknown_field = "value"
        "#,
        );
        assert!(result.is_err());
    }
}
