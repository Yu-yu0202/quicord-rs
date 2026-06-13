use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_logger() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = fmt::layer()
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false);

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
