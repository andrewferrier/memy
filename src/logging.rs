use std::io::stderr;
use std::io::IsTerminal as _;
use tracing_log::LogTracer;
use tracing_subscriber::{fmt, EnvFilter};

pub fn configure_logging_and_tracing(verbose: u8) {
    LogTracer::init().expect("Failed to init LogTracer");

    let default_level = match verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    // RUST_LOG overrides --verbose if set
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    let subscriber = fmt::Subscriber::builder()
        .with_env_filter(env_filter)
        .with_ansi(stderr().is_terminal())
        .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT)
        .event_format(fmt::format().compact())
        .with_writer(std::io::stderr)
        .without_time()
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
