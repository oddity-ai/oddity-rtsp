mod app;
mod net;
mod media;
mod source;
mod session;
mod runtime;

use std::path::Path;
use std::error::Error;
use std::env;
use std::process;

use config::ConfigError;

use app::App;
use app::config::AppConfig;

use tokio::signal::ctrl_c;

use video_rs as video;

macro_rules! on_error_exit {
  ($expr:expr) => {
    match $expr {
      Ok(ret) => ret,
      Err(err) => {
        println!("\x1b[1m\x1b[91mError:\x1b[0m {}", err);
        process::exit(1);
      },
    }
  };
}

#[tokio::main]
async fn main() {
  on_error_exit!(initialize_tracing());
  initialize_media();

  let config = on_error_exit!(initialize_and_read_config());
  tracing::debug!(?config, "loaded config file");

  tracing::trace!("starting app");
  let mut app = on_error_exit!(App::start(config).await);
  tracing::trace!("started app");

  tracing::trace!("waiting for ctrl+C...");
  on_error_exit!(ctrl_c().await);

  tracing::trace!("stopping app");
  app.stop().await;
  tracing::trace!("stopped app");
}

fn initialize_tracing() -> Result<(), Box<dyn Error + Send + Sync>> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .try_init()
}

fn initialize_media() {
  video::init();
}

fn initialize_and_read_config() -> Result<AppConfig, ConfigError> {
  let config_file = env::args()
    .nth(1)
    .unwrap_or("default.yaml".to_string());
  let config_file = Path::new(&config_file);
  tracing::trace!(config_file=%config_file.display(), "loading config");

  AppConfig::from_file(config_file)
}