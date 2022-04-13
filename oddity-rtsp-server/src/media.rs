use std::fmt;
use std::collections::HashMap;

use super::multiplexer::Multiplexer;

pub enum Source {
  Multiplex(Multiplexer)
}

impl fmt::Display for Source {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Source::Multiplex(multiplexer) => multiplexer.fmt(f),
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

  pub fn query_source_sdp(
    &self,
    path: &str,
  ) -> Option<String> {
    // TODO
    None
  }

  // TODO Implement
  // pub fn play(
  //   &self,
  //   session,
  //   path: &str,
  // ) -> Result {
  // }

  // TODO Implement
  // pub fn pause(
  //   &self,
  //   session,
  //   path: &str,
  // ) -> Result {
  // }

  // TODO Implement
  // // TODO If possible we should just make this a no-op and
  // //   do all cleaning up automagically.
  // pub fn cleanup(
  //   &self,
  //   session,
  //   path: &str,
  // ) -> Result {
  // }

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