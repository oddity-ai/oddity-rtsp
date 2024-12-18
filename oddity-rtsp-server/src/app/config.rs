use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use config::{Config, ConfigError};

use crate::media::MediaDescriptor;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: Server,
    pub media: Vec<Item>,
}

#[derive(Debug, Deserialize)]
pub struct Server {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct Item {
    pub name: String,
    pub path: String,
    pub kind: MediaKind,
    pub source: String,
}

impl Item {
    pub fn as_media_descriptor(&self) -> Result<MediaDescriptor, Box<dyn Error>> {
        Ok(match self.kind {
            MediaKind::File => MediaDescriptor::File(PathBuf::from(self.source.to_string())),
            MediaKind::Stream => MediaDescriptor::Stream(self.source.parse()?),
        })
    }

    fn source_safe_display(&self) -> String {
        if matches!(self.kind, MediaKind::Stream) {
            video_rs::Url::parse(&self.source)
                .map(|url| {
                    let mut url_safe = url.clone();
                    let _ = url_safe.set_password(None);
                    url_safe.to_string()
                })
                .unwrap_or_else(|_| "<invalid url>".to_string())
        } else {
            self.source.to_string()
        }
    }
}

impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} ({}): {} ({})",
            self.name,
            self.path,
            self.source_safe_display(),
            self.kind,
        )
    }
}

impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ name: {:?}, path: {:?}, kind: {:?}, source: {:?} }}",
            self.name,
            self.path,
            self.kind,
            self.source_safe_display(),
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    File,
    Stream,
}

impl fmt::Display for MediaKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaKind::File => write!(f, "file"),
            MediaKind::Stream => write!(f, "live stream"),
        }
    }
}

impl Default for AppConfig {
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

impl AppConfig {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        Config::builder()
            .add_source(config::File::from(path))
            .add_source(config::Environment::with_prefix("oddity"))
            .build()?
            .try_deserialize()
    }
}
