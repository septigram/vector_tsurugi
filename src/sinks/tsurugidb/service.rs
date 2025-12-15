use tsubakuro_rust_core::prelude::*;

use std::{
    num::NonZeroUsize,
    sync::Arc,
    task::{Context, Poll},
};

use futures::future::BoxFuture;
use snafu::{ResultExt, Snafu};
use tower::Service;
use vector_lib::{
    EstimatedJsonEncodedSizeOf,
    codecs::JsonSerializerConfig,
    event::{Event, EventFinalizers, EventStatus, Finalizable},
    request_metadata::{GroupedCountByteSize, MetaDescriptive, RequestMetadata},
    stream::DriverResponse,
};

use crate::{
    internal_events::EndpointBytesSent,
    sinks::prelude::{RequestMetadataBuilder, RetryLogic},
};

const TSURUGI_PROTOCOL: &str = "tsurugi";

/// JSONオブジェクトからINSERT文を生成
fn build_insert_sql(table: &str, value: &serde_json::Value) -> Result<String, serde_json::Error> {
    let obj = value.as_object().ok_or_else(|| {
        // serde_json::Errorを作成するために、io::Errorを使用
        serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Expected JSON object for INSERT statement",
        ))
    })?;

    let mut columns = Vec::new();
    let mut values = Vec::new();

    for (key, val) in obj {
        // timestampフィールドは予約語の可能性があるため、スキップ
        // （event_timestampフィールドが既に存在する場合、重複を避けるため）
        if key == "timestamp" {
            continue;
        }
        columns.push(key.clone());
        // event_timestampフィールドの場合、TIMESTAMP型用の形式に変換
        let sql_value = if key == "event_timestamp" {
            json_value_to_timestamp_literal(val)?
        } else {
            json_value_to_sql_literal(val)?
        };
        values.push(sql_value);
    }

    if columns.is_empty() {
        return Err(serde_json::Error::io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Cannot create INSERT with no columns",
        )));
    }

    let sql = format!(
        "INSERT INTO {} ({}) VALUES ({})",
        table,
        columns.join(", "),
        values.join(", ")
    );

    Ok(sql)
}

/// TIMESTAMP型用の値をSQLリテラルに変換
/// TsurugiDBのTIMESTAMP型は 'YYYY-MM-DD HH:MM:SS' 形式を期待
fn json_value_to_timestamp_literal(value: &serde_json::Value) -> Result<String, serde_json::Error> {
    match value {
        serde_json::Value::String(s) => {
            // ISO 8601形式の文字列をパースして、TsurugiDB形式に変換
            // 例: '2025-12-15T05:34:01.000904Z' -> '2025-12-15 05:34:01'
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(s) {
                let formatted = dt.format("%Y-%m-%d %H:%M:%S").to_string();
                Ok(format!("'{}'", formatted))
            } else {
                // パースに失敗した場合は、そのまま使用（エラーになる可能性がある）
                let escaped = s.replace('\'', "''");
                Ok(format!("'{}'", escaped))
            }
        }
        _ => {
            // 文字列以外の場合は、通常の変換を使用
            json_value_to_sql_literal(value)
        }
    }
}

/// JSONの値をSQLリテラルに変換
fn json_value_to_sql_literal(value: &serde_json::Value) -> Result<String, serde_json::Error> {
    match value {
        serde_json::Value::Null => Ok("NULL".to_string()),
        serde_json::Value::Bool(b) => Ok(b.to_string()),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                Ok(n.as_i64().unwrap().to_string())
            } else if n.is_u64() {
                Ok(n.as_u64().unwrap().to_string())
            } else {
                Ok(n.as_f64().unwrap().to_string())
            }
        }
        serde_json::Value::String(s) => {
            // SQLインジェクション対策: シングルクォートをエスケープ
            let escaped = s.replace('\'', "''");
            Ok(format!("'{}'", escaped))
        }
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            // 配列やオブジェクトはJSON文字列として扱う
            let json_str = serde_json::to_string(value)?;
            let escaped = json_str.replace('\'', "''");
            Ok(format!("'{}'", escaped))
        }
    }
}

#[derive(Clone)]
pub struct TsurugiRetryLogic;

impl RetryLogic for TsurugiRetryLogic {
    type Error = TsurugiServiceError;
    type Request = TsurugiRequest;
    type Response = TsurugiResponse;

    fn is_retriable_error(&self, error: &Self::Error) -> bool {
        match error {
            TsurugiServiceError::Tsurugi { message } => {
                // エラーメッセージから判定
                // I/Oエラー、タイムアウトエラー、接続関連のエラーはリトライ可能
                message.contains("IoError") 
                    || message.contains("TimeoutError")
                    || message.contains("connection")
                    || message.contains("network")
                    || message.contains("ServerError") // 暫定: サーバーエラーもリトライ可能とする
            }
            TsurugiServiceError::VectorCommon { .. } => false,
            TsurugiServiceError::JsonSerialization { .. } => false,
        }
    }
}

#[derive(Clone)]
pub struct TsurugiService {
    session: Arc<Session>,
    table: String,
    endpoint: String,
    transaction_type: TransactionType,
}

impl TsurugiService {
    pub fn new(
        session: Arc<Session>,
        table: String,
        endpoint: String,
        transaction_type: TransactionType,
    ) -> Self {
        Self {
            session,
            table,
            endpoint,
            transaction_type,
        }
    }
}

#[derive(Clone)]
pub struct TsurugiRequest {
    pub events: Vec<Event>,
    pub finalizers: EventFinalizers,
    pub metadata: RequestMetadata,
}

impl TryFrom<Vec<Event>> for TsurugiRequest {
    type Error = String;

    fn try_from(mut events: Vec<Event>) -> Result<Self, Self::Error> {
        let finalizers = events.take_finalizers();
        let metadata_builder = RequestMetadataBuilder::from_events(&events);
        let events_size = NonZeroUsize::new(events.estimated_json_encoded_size_of().get())
            .ok_or("payload should never be zero length")?;
        let metadata = metadata_builder.with_request_size(events_size);
        Ok(TsurugiRequest {
            events,
            finalizers,
            metadata,
        })
    }
}

impl Finalizable for TsurugiRequest {
    fn take_finalizers(&mut self) -> EventFinalizers {
        self.finalizers.take_finalizers()
    }
}

impl MetaDescriptive for TsurugiRequest {
    fn get_metadata(&self) -> &RequestMetadata {
        &self.metadata
    }

    fn metadata_mut(&mut self) -> &mut RequestMetadata {
        &mut self.metadata
    }
}

pub struct TsurugiResponse {
    metadata: RequestMetadata,
}

impl DriverResponse for TsurugiResponse {
    fn event_status(&self) -> EventStatus {
        EventStatus::Delivered
    }

    fn events_sent(&self) -> &GroupedCountByteSize {
        self.metadata.events_estimated_json_encoded_byte_size()
    }

    fn bytes_sent(&self) -> Option<usize> {
        Some(self.metadata.request_encoded_size())
    }
}

#[derive(Debug, Snafu)]
pub enum TsurugiServiceError {
    #[snafu(display("Tsurugi database error: {message}"))]
    Tsurugi { message: String },

    #[snafu(display("Serialization error: {source}"))]
    VectorCommon { source: vector_common::Error },

    #[snafu(display("JSON serialization error: {source}"))]
    JsonSerialization { source: serde_json::Error },
}

impl Service<TsurugiRequest> for TsurugiService {
    type Response = TsurugiResponse;
    type Error = TsurugiServiceError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: TsurugiRequest) -> Self::Future {
        let service = self.clone();
        let future = async move {
            // 1. SqlClientの取得
            let client: SqlClient = service.session.make_client();
            
            // 2. トランザクションの開始
            let mut transaction_option = TransactionOption::new();
            transaction_option.set_transaction_type(service.transaction_type);
            let transaction = client.start_transaction(&transaction_option).await
                .map_err(|e| TsurugiServiceError::Tsurugi { 
                    message: format!("{}", e) 
                })?;
            
            // 3. イベントのシリアライズ
            let json_serializer = JsonSerializerConfig::default().build();
            let serialized_values: Vec<serde_json::Value> = request
                .events
                .into_iter()
                .map(|event| json_serializer.to_json_value(event))
                .collect::<Result<Vec<_>, _>>()
                .context(VectorCommonSnafu)?;
            
            // 4. 個別INSERT文の生成と実行
            // 注意: TsurugiはPostgreSQLのjsonb_populate_recordset相当の機能を
            // サポートしていないため、各イベントを個別のINSERT文として実行する
            for value in serialized_values {
                // JSONオブジェクトからINSERT文を構築
                // 例: {"col1": "val1", "col2": 123} -> INSERT INTO table (col1, col2) VALUES ('val1', 123)
                let sql = build_insert_sql(&service.table, &value)
                    .context(JsonSerializationSnafu)?;
                
                client.execute(&transaction, &sql).await
                    .map_err(|e| TsurugiServiceError::Tsurugi { 
                        message: format!("{}", e) 
                    })?;
            }
            
            // 5. コミット
            let commit_option = CommitOption::default();
            client.commit(&transaction, &commit_option).await
                .map_err(|e| TsurugiServiceError::Tsurugi { 
                    message: format!("{}", e) 
                })?;
            
            // 6. トランザクションのクローズ
            transaction.close().await
                .map_err(|e| TsurugiServiceError::Tsurugi { 
                    message: format!("{}", e) 
                })?;
            
            // 7. メトリクスの発行
            emit!(EndpointBytesSent {
                byte_size: request.metadata.request_encoded_size(),
                protocol: TSURUGI_PROTOCOL,
                endpoint: &service.endpoint,
            });
            
            Ok(TsurugiResponse {
                metadata: request.metadata,
            })
        };
        
        Box::pin(future)
    }
}
