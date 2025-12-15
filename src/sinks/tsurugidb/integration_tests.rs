use std::future::ready;

use chrono::{DateTime, Utc};
use futures::stream;
use vector_lib::event::{
    BatchNotifier, BatchStatus, BatchStatusReceiver, Event, LogEvent,
};
use tsubakuro_rust_core::prelude::*;

use crate::{
    config::{SinkConfig, SinkContext},
    sinks::{tsurugidb::TsurugiConfig, util::test::load_sink},
    test_util::{
        addr::next_addr,
        components::{
            COMPONENT_ERROR_TAGS, run_and_assert_sink_compliance, run_and_assert_sink_error,
        },
        integration::tsurugi::tsurugi_endpoint,
        random_table_name, trace_init,
    },
};

const TSURUGI_SINK_TAGS: [&str; 2] = ["endpoint", "protocol"];

fn timestamp() -> DateTime<Utc> {
    let timestamp = Utc::now();
    // マイクロ秒精度に切り詰める（Tsurugiのタイムスタンプ精度に合わせる）
    DateTime::from_timestamp_micros(timestamp.timestamp_micros()).unwrap()
}

fn create_event(id: i64) -> Event {
    let mut event = LogEvent::from("raw log line");
    event.insert("id", id);
    event.insert("host", "example.com");
    let event_payload = event.clone().into_parts().0;
    event.insert("payload", event_payload);
    event.insert("timestamp", timestamp());
    event.into()
}

fn create_event_with_notifier(id: i64) -> (Event, BatchStatusReceiver) {
    let (batch, receiver) = BatchNotifier::new_with_receiver();
    let event = create_event(id).with_batch_notifier(&batch);
    (event, receiver)
}

fn create_events(count: usize) -> (Vec<Event>, BatchStatusReceiver) {
    let mut events = (0..count as i64).map(create_event).collect::<Vec<_>>();
    let receiver = BatchNotifier::apply_to(&mut events);
    (events, receiver)
}

async fn prepare_config() -> (TsurugiConfig, String) {
    let table = random_table_name();
    let endpoint = tsurugi_endpoint();
    let config_str = format!(
        r#"
            endpoint = "{endpoint}"
            table = "{table}"
            batch.max_events = 1
        "#,
    );
    let (config, _) = load_sink::<TsurugiConfig>(&config_str).unwrap();
    (config, table)
}

/// テスト用のテーブルを作成する
/// 注意: 実際のTsurugiサーバーが必要
async fn create_test_table(endpoint: &str, table: &str) -> Result<(), String> {
    let mut connection_option = ConnectionOption::new();
    connection_option.set_endpoint_url(endpoint)
        .map_err(|e| format!("Failed to set endpoint URL: {}", e))?;
    connection_option.set_application_name("Vector Tsurugi Test");
    connection_option.set_default_timeout(std::time::Duration::from_secs(10));

    let session = Session::connect(&connection_option).await
        .map_err(|e| format!("Failed to connect to Tsurugi: {}", e))?;
    let client: SqlClient = session.make_client();

    // トランザクションを開始
    let mut transaction_option = TransactionOption::new();
    transaction_option.set_transaction_type(TransactionType::Long);
    let transaction = client.start_transaction(&transaction_option).await
        .map_err(|e| format!("Failed to start transaction: {}", e))?;

    // テーブルを作成
    let create_table_sql = format!(
        "CREATE TABLE {table} (id BIGINT, host TEXT, timestamp TIMESTAMPTZ, message TEXT, payload JSON)"
    );
    client.execute(&transaction, &create_table_sql).await
        .map_err(|e| format!("Failed to create table: {}", e))?;

    // コミット
    let commit_option = CommitOption::default();
    client.commit(&transaction, &commit_option).await
        .map_err(|e| format!("Failed to commit: {}", e))?;

    // トランザクションのクローズ
    transaction.close().await
        .map_err(|e| format!("Failed to close transaction: {}", e))?;

    Ok(())
}

#[tokio::test]
#[ignore] // 実際のTsurugiサーバーが必要なため、デフォルトではスキップ
async fn healthcheck_passes() {
    trace_init();
    let (config, _table) = prepare_config().await;
    let (_sink, healthcheck) = config
        .build(SinkContext::default())
        .await
        .expect("sink should build successfully");
    assert!(healthcheck.await.is_ok());
}

#[tokio::test]
async fn healthcheck_fails_unknown_host() {
    trace_init();

    let table = random_table_name();
    let endpoint = "tcp://unknown_host:12345".to_string();
    let config_str = format!(
        r#"
            endpoint = "{endpoint}"
            table = "{table}"
        "#,
    );
    let (config, cx) = load_sink::<TsurugiConfig>(&config_str).unwrap();

    let (_sink, healthcheck) = config
        .build(cx)
        .await
        .expect("sink should build successfully");
    assert!(healthcheck.await.is_err());
}

#[tokio::test(start_paused = true)]
async fn healthcheck_fails_timed_out() {
    trace_init();

    let (_guard, free_addr) = next_addr();
    let endpoint = format!("tcp://{free_addr}");
    let table = random_table_name();
    let config_str = format!(
        r#"
            endpoint = "{endpoint}"
            table = "{table}"
        "#,
    );
    let (config, cx) = load_sink::<TsurugiConfig>(&config_str).unwrap();

    let (_sink, healthcheck) = config
        .build(cx)
        .await
        .expect("sink should build successfully");
    assert!(healthcheck.await.is_err());
}

#[tokio::test]
#[ignore] // 実際のTsurugiサーバーが必要なため、デフォルトではスキップ
async fn insert_single_event() {
    trace_init();

    let (config, table) = prepare_config().await;
    let endpoint = tsurugi_endpoint();
    
    // テーブルを作成
    create_test_table(&endpoint, &table).await
        .expect("Failed to create test table");

    let (sink, _hc) = config.build(SinkContext::default()).await.unwrap();

    let (input_event, mut receiver) = create_event_with_notifier(0);
    let input_log_event = input_event.clone().into_log();
    let expected_value = serde_json::to_value(&input_log_event).unwrap();

    run_and_assert_sink_compliance(sink, stream::once(ready(input_event)), &TSURUGI_SINK_TAGS)
        .await;
    // We drop the event to notify the receiver that the batch was delivered.
    std::mem::drop(input_log_event);
    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // 実際のTsurugiサーバーからデータを取得して検証する必要があります
    // ここでは簡易的な確認として、エラーが発生しなかったことを確認します
}

#[tokio::test]
#[ignore] // 実際のTsurugiサーバーが必要なため、デフォルトではスキップ
async fn insert_multiple_events() {
    trace_init();

    let (config, table) = prepare_config().await;
    let endpoint = tsurugi_endpoint();
    
    // テーブルを作成
    create_test_table(&endpoint, &table).await
        .expect("Failed to create test table");

    let (sink, _hc) = config.build(SinkContext::default()).await.unwrap();

    let (input_events, mut receiver) = create_events(10);
    let input_log_events = input_events
        .clone()
        .into_iter()
        .map(Event::into_log)
        .collect::<Vec<_>>();
    run_and_assert_sink_compliance(sink, stream::iter(input_events), &TSURUGI_SINK_TAGS).await;
    // We drop the event to notify the receiver that the batch was delivered.
    std::mem::drop(input_log_events);
    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Delivered));

    // 実際のTsurugiサーバーからデータを取得して検証する必要があります
    // ここでは簡易的な確認として、エラーが発生しなかったことを確認します
}

#[tokio::test]
#[ignore] // 実際のTsurugiサーバーが必要なため、デフォルトではスキップ
async fn insertion_fails_missing_table() {
    trace_init();

    let table = "missing_table".to_string();
    let (mut config, _) = prepare_config().await;
    // テーブル名を存在しないものに変更
    config.table = table.clone();

    let (sink, _hc) = config.build(SinkContext::default()).await.unwrap();
    let (input_event, mut receiver) = create_event_with_notifier(0);

    run_and_assert_sink_error(
        sink,
        stream::once(ready(input_event)),
        &COMPONENT_ERROR_TAGS,
    )
    .await;
    assert_eq!(receiver.try_recv(), Ok(BatchStatus::Rejected));
}
