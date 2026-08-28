use telemetry_export_receipts::{app, config::Config};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "telemetry_export_receipts=info,tower_http=info".into()),
        )
        .init();
    let config = Config::from_env().unwrap_or_else(|message| {
        eprintln!("configuration error: {message}");
        std::process::exit(2);
    });
    let address = format!("0.0.0.0:{}", config.port);
    let router = app::build(config)
        .await
        .expect("database initialization failed");
    let listener = TcpListener::bind(&address).await.expect("port bind failed");
    tracing::info!(%address, "server ready");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown())
        .await
        .expect("server failed");
}

async fn shutdown() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("ctrl-c handler") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("signal handler")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
}
