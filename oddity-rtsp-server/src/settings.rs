use std::path::Path;

use serde::Deserialize;

use config::{
  Config,
  ConfigError,
  Environment,
  File,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
  Multiplex,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub server: Server,
  pub media: Vec<MediaItem>,
}

#[derive(Debug, Deserialize)]
pub struct Server {
  pub host: String,
  pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct MediaItem {
  pub kind: MediaKind,
  pub path: String,
  pub uri: String,
}

impl Default for Settings {

  fn default() -> Self {
    Self {
      server: Server {
        host: "127.0.0.1".to_string(),
        port: 554,
      },
      media: Vec::new(),
    }
  }

}

impl Settings {

  pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
    Config::builder()
      .add_source(File::from(path))
      .add_source(Environment::with_prefix("oddity"))
      .build()?
      .try_deserialize()
  }

}