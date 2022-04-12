use std::fmt;
use std::collections::HashMap;

use oddity_rtsp_protocol::Uri;

pub enum Source {
  Multiplex(Multiplexer)
}

impl fmt::Display for Source {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Source::Multiplex(Multiplexer { uri }) =>
        write!(f, "multiplex from source: {}", uri),
    }
  }

}

pub struct Multiplexer {
  /// Source URI from which to read and multiplex stream.
  uri: Uri,
}

impl Multiplexer {

  pub fn new(uri: Uri) -> Multiplexer {
    Self {
      uri,
    }
  }

}

pub struct MediaController {
  store: HashMap<String, Source>,
}

impl MediaController {

  pub fn new() -> Self {
    Self {
      store: Default::default(),
    }
  }

  pub fn register_source(
    &mut self,
    path: &str,
    source: Source,
  ) {
    let _ = self.store.insert(path.to_string(), source);
  }

}

impl fmt::Display for MediaController {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "Registered sources:")?;
    for (var, val) in self.store.iter() {
      writeln!(f, " - {}: {}", var, val)?;
    }
    Ok(())
  }

}