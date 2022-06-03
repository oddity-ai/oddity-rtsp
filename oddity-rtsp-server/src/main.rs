mod app;
mod net;
mod media;
mod source;
mod session;
mod runtime;

use std::path::Path;
use std::error::Error;
use std::env;

use config::ConfigError;

use app::App;
use app::config::AppConfig;

use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
  initialize_tracing()
    .expect("failed to initialize tracing");

  let config =
    initialize_and_read_config()
      .expect("failed to load config");
  tracing::debug!(?config, "loaded config file");

  tracing::trace!("starting app");
  let mut app = App::start(config)
    .await
    .expect("failed to start application");
  tracing::trace!("started app");

  tracing::trace!("waiting for ctrl+C...");
  // Wait for SIGINT and then stop the application.
  ctrl_c()
    .await
    .expect("failed to listen for signal");

  tracing::trace!("stopping app");
  app.stop().await;
  tracing::trace!("stopped app");
}

fn initialize_tracing() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .try_init()
}

fn initialize_and_read_config() -> Result<AppConfig, ConfigError> {
  let config_file = env::args()
    .nth(1)
    .unwrap_or("default.yaml".to_string());
  let config_file = Path::new(&config_file);
  tracing::trace!(config_file=%config_file.display(), "loading config");

  AppConfig::from_file(config_file)
}