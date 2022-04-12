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
pub struct MediaItem {
  pub kind: MediaKind,
  pub path: String,
  pub uri: String,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
  pub media: Vec<MediaItem>,
}

impl Default for Settings {

  fn default() -> Self {
    Self {
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