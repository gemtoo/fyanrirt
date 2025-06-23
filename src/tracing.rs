use colored::Colorize;
use tracing_subscriber;
use tracing_subscriber::EnvFilter;

pub fn init() {
    let filter = EnvFilter::new("fyanrirt=trace");
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_env_filter(filter)
        //.with_span_events(FmtSpan::CLOSE)
        //.with_thread_names(false)
        .with_target(false)
        //.with_thread_ids(false)
        //.with_ansi(true)
        //.compact()
        .init();
    info!(
        "{}",
        "________                            _____       _____    ".green()
    );
    info!(
        "{}",
        "___  __/____  _______ _________________(_)________  /_   ".green()
    );
    info!(
        "{}",
        "__  /_ __  / / /  __ `/_  __ \\_  ___/_  /__  ___/  __/  ".green()
    );
    info!(
        "{}",
        "_  __/ _  /_/ // /_/ /_  / / /  /   _  / _  /   / /_     ".green()
    );
    info!(
        "{}",
        "/_/    _\\__, / \\__,_/ /_/ /_//_/    /_/  /_/    \\__/  ".green()
    );
    info!(
        "{}",
        "       /____/                                            ".green()
    );
    let version = env!("CARGO_PKG_VERSION");
    info!("Started up. Version: {}", version);
}
