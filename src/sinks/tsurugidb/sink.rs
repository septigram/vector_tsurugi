use super::service::{TsurugiRequest, TsurugiRetryLogic, TsurugiService};
use crate::sinks::prelude::*;

pub struct TsurugiSink {
    service: Svc<TsurugiService, TsurugiRetryLogic>,
    batch_settings: BatcherSettings,
}

impl TsurugiSink {
    pub const fn new(
        service: Svc<TsurugiService, TsurugiRetryLogic>,
        batch_settings: BatcherSettings,
    ) -> Self {
        Self {
            service,
            batch_settings,
        }
    }

    async fn run_inner(self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        input
            .batched(self.batch_settings.as_byte_size_config())
            .filter_map(|events| async move {
                match TsurugiRequest::try_from(events) {
                    Ok(request) => Some(request),
                    Err(e) => {
                        warn!(
                            message = "Error creating tsurugi sink's request.",
                            error = %e
                        );
                        None
                    }
                }
            })
            .into_driver(self.service)
            .run()
            .await
    }
}

#[async_trait::async_trait]
impl StreamSink<Event> for TsurugiSink {
    async fn run(mut self: Box<Self>, input: BoxStream<'_, Event>) -> Result<(), ()> {
        self.run_inner(input).await
    }
}
