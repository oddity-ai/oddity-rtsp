mod server;
mod connection;
mod media;
mod settings;
mod transport;

use std::error::Error;
use std::env::args;
use std::path::Path;

use server::Server;
use media::{Controller as MediaController, Descriptor};
use settings::{Settings, MediaKind};

fn main() {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .init();

  let settings_file = args()
    .nth(1)
    .unwrap_or("default.yaml".to_string());
  let settings_file = Path::new(&settings_file);

  let settings = Settings::from_file(settings_file)?;
  tracing::debug!(?settings, "read settings file");

  let mut media_controller = MediaController::new();
  for media_item in settings.media.iter() {
    let descriptor = match media_item.kind {
      MediaKind::File
        => Descriptor::File(media_item.source.as_str().into()),
      MediaKind::Stream
        => Descriptor::Stream(media_item.source.parse()?),
    };

    media_controller.register_source(&media_item.path, &descriptor);
    tracing::info!(%descriptor, "registered media item");
  }

  tracing::info!(%media_controller, "initialized media controller");

  Server::new(
      (settings.server.host, settings.server.port),
      media_controller
    )
    .run()
    .await
}