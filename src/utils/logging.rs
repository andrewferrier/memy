use tracing_log::LogTracer;
use tracing_subscriber::{EnvFilter, fmt};

pub fn configure_logging_and_tracing(verbose: u8, color: Option<bool>) {
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

    let subscriber = {
        let mut builder = fmt::Subscriber::builder()
            .with_env_filter(env_filter)
            .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT)
            .event_format(fmt::format().compact())
            .with_writer(std::io::stderr)
            .without_time();

        if let Some(value) = color {
            builder = builder.with_ansi(value);
        }

        builder.finish()
    };

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}
