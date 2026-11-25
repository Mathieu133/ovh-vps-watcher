mod config;
mod discord;
mod ovh;
mod watcher;

use std::time::Duration;

use anyhow::Result;
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use crate::{config::Config, discord::DiscordWebhook, ovh::OvhClient, watcher::Watcher};

const HTTP_TIMEOUT: Duration = Duration::from_secs(15);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let config = Config::from_env()?;

    let http = reqwest::Client::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(HTTP_TIMEOUT)
        .build()?;

    let ovh = OvhClient::new(http.clone(), config.subsidiary.clone());
    let discord = DiscordWebhook::new(http, config.webhook_url.clone());

    let watched_models = config
        .models
        .iter()
        .map(u8::to_string)
        .collect::<Vec<_>>()
        .join("/");

    info!(
        models = %watched_models,
        interval_secs = config.check_interval.as_secs(),
        "watcher started"
    );

    if config.send_startup_message {
        let message = format!("👀 Surveillance des VPS-{watched_models} OVH (Europe) démarrée.");
        if let Err(err) = discord.send(&message).await {
            warn!(error = %err, "failed to send startup message");
        }
    }

    let watcher = Watcher::new(ovh, discord, config.models);

    tokio::select! {
        () = watcher.run(config.check_interval) => {}
        () = shutdown_signal() => info!("shutting down"),
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {}
        () = terminate => {}
    }
}
