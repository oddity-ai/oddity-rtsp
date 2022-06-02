mod app;
mod settings;
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
use settings::Settings;

use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
  initialize_tracing()
    .expect("failed to initialize tracing");

  let settings =
    initialize_and_read_settings()
      .expect("failed to load settings");

  tracing::debug!(?settings, "read settings file");

  let mut app = App::start().await;
  ctrl_c().await.expect("failed to listen for signal");
  app.stop().await;
}

fn initialize_tracing() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .try_init()
}

fn initialize_and_read_settings() -> Result<Settings, ConfigError> {
  let settings_file = env::args()
    .nth(1)
    .unwrap_or("default.yaml".to_string());
  let settings_file = Path::new(&settings_file);

  Settings::from_file(settings_file)
}