use tracing::Subscriber;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::SubscriberExt;

static BUNYAN_FORMATTER_NAME: &str = "list-management";

/*
This is adopted from:
https://github.com/LukeMathWalker/zero-to-production/blob/main/src/telemetry.rs
*/
pub fn tracing_subscriber<Sink>(
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Sync + Send
    where
        Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(BUNYAN_FORMATTER_NAME.to_string(), sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}