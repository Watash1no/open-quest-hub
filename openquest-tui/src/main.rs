use std::panic;

fn main() {
    // Setup panic handler
    panic::set_hook(Box::new(|panic_info| {
        eprintln!("[PANIC] {}", panic_info);
        if let Some(location) = panic_info.location() {
            eprintln!("  at {}:{}:{}", location.file(), location.line(), location.column());
        }
    }));

    // Initialize logging
    let log_dir = directories::ProjectDirs::from("com", "openquest", "tui")
        .map(|d| d.data_local_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    std::fs::create_dir_all(&log_dir).ok();

    let file_appender = tracing_appender::rolling::daily(&log_dir, "openquest-tui.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .init();

    tracing::info!("Starting OpenQuest TUI v1.0.0");

    // Run with tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    if let Err(e) = runtime.block_on(openquest_tui::run()) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}