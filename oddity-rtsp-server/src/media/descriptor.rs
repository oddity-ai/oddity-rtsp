use std::path::PathBuf;
use std::fmt;

use oddity_video::{Locator, Url};

#[derive(Clone)]
pub enum Descriptor {
  Stream(Url),
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
      Descriptor::Stream(url)
        => Locator::Url(url),
    }
  }

}