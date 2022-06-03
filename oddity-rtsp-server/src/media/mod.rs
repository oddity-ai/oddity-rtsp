pub mod sdp;
pub mod video;

use oddity_video::StreamInfo;

pub use oddity_video::Packet;

use std::path::PathBuf;
use std::fmt;

use oddity_video::{Locator, Url};

#[derive(Clone)]
pub enum MediaDescriptor {
  Stream(Url),
  File(PathBuf),
}

impl fmt::Display for MediaDescriptor {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MediaDescriptor::File(path) =>
        write!(f, "file: {}", path.display()),
      MediaDescriptor::Stream(url) =>
        write!(f, "stream: {}", url),
    }
  }

}

impl From<MediaDescriptor> for Locator {

  fn from(descriptor: MediaDescriptor) -> Self {
    match descriptor {
      MediaDescriptor::File(path)
        => Locator::Path(path.into()),
      MediaDescriptor::Stream(url)
        => Locator::Url(url),
    }
  }

}

#[derive(Clone)]
pub struct MediaInfo {
  streams: Vec<StreamInfo>,
}