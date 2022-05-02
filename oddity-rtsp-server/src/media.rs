mod source;
mod session;
mod controller;

use std::path::PathBuf;
use std::fmt;

use oddity_rtsp_protocol::Uri;
use oddity_video::Locator;

pub use controller::Controller;
pub use source::Source;
pub use session::{
  SessionId,
  Session,
};

#[derive(Clone)]
pub enum Descriptor {
  Stream(Uri),
  File(PathBuf),
}

impl fmt::Display for Descriptor {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Descriptor::File(path) =>
        write!(f, "file: {}", path.display()),
      Descriptor::Stream(url) =>
        write!(f, "stream: {}", url),
    }
  }

}

impl From<Descriptor> for Locator {

  fn from(descriptor: Descriptor) -> Self {
    match descriptor {
      Descriptor::File(path)
        => Locator::Path(path.into()),
      Descriptor::Stream(uri)
        => Locator::Url(uri.into()),
    }
  }

}

#[derive(Debug)]
pub enum State {
  Init,
  Playing,
  // TODO See RFC
}

impl fmt::Display for State {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      State::Init => write!(f, "initialization"),
      State::Playing => write!(f, "playing"),
    }
  }

}