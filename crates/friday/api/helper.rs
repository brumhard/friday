use aide::openapi::{Info, OpenApi};
use tokio::signal;
use tracing::Level;
use tracing_subscriber::prelude::*;

pub fn enable_tracing() {
    // https://stackoverflow.com/questions/74302133/how-to-log-and-filter-requests-with-axum-tokio
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("tower_http::trace::on_response", Level::DEBUG)
        .with_target("tower_http::trace::on_request", Level::DEBUG)
        .with_target("tower_http::trace::make_span", Level::DEBUG)
        .with_default(Level::INFO);
    let tracing_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .init();
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");
}

pub fn openapi_spec() -> OpenApi {
    OpenApi {
        info: Info {
            description: Some("Friday API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    }
}
