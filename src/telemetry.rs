use tokio::task::JoinHandle;
use tracing::Subscriber;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::fmt::time::ChronoLocal;
use tracing_subscriber::fmt::{self, MakeWriter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

// We are using `impl Subscriber` as return type to avoid having to
/// spell out the actual type of the returned subscriber, which is
/// indeed quite complex.
/// We need to explicitly call out that the returned subscriber is
/// `Send` and `Sync` to make it possible to pass it to `init_subscriber`
/// later on.
pub fn get_subscriber<Sink>(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    // This "weired" syntax is a higher-ranked trait bound(HRTB)
    // It basically means that Sink implements the `MakerWriter`
    // trait for all choices of the lifetime parameter `'a`
    // chec kout https://doc.rust-lang.org/nomicon/hrtb.html
    // for more details.
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    let time_fmt_layer = fmt::Layer::default()
        .with_target(false)
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()));
    Registry::default()
        .with(env_filter)
        .with(time_fmt_layer)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

/// Register a subscriber as global default to process span data.
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

pub fn spawn_blocking_with_tracing<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let current_span = tracing::Span::current();
    tokio::task::spawn_blocking(move || current_span.in_scope(f))
}
