use std::fmt;
use std::collections::{HashMap, hash_map::Entry};

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

pub struct Session<'session> {
  state: State,
  source: &'session mut Source,
}

impl Session<'_> {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate_id() -> String {
    rand::thread_rng()
      .sample_iter(&rand::distributions::Alphanumeric)
      .take(Self::SESSION_ID_LEN)
      .map(char::from)
      .collect()
  }

}

impl fmt::Display for Session<'_> {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{} ({})", self.state, self.source)
  }

}

#[derive(Debug)]
pub enum State {
  Init,
  Playing,
}

impl fmt::Display for State {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      State::Init => write!(f, "initialization"),
      State::Playing => write!(f, "playing"),
    }
  }

}

pub struct MediaController<'scope> {
  sources: HashMap<String, Source>,
  sessions: HashMap<String, Session<'scope>>,
}

impl<'scope> MediaController<'scope> {

  pub fn new() -> Self {
    Self {
      sources: Default::default(),
      sessions: Default::default(),
    }
  }

  pub fn register_source(
    &mut self,
    path: &str,
    source: Source,
  ) {
    let _ = self.sources.insert(path.to_string(), source);
  }

  pub fn query_source_sdp(
    &self,
    path: &str,
  ) -> Option<String> {
    // TODO
    None
  }

  pub fn register_session(
    &mut self,
    path: &str,
  ) -> Result<(String, &'scope Session<'scope>), RegisterSessionError> {
    if let Some(source) = self.sources.get_mut(path) {
      let session_id = Session::generate_id();
      if let Entry::Vacant(entry) = self.sessions.entry(session_id) {
        entry.insert(Session {
          state: State::Init,
          source,
        })
      } else {
        Err(RegisterSessionError::AlreadyExists)
      }
    } else {
      Err(RegisterSessionError::NotFound)
    }
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

impl fmt::Display for MediaController<'_> {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "sources:")?;
    for (path, source) in self.sources.iter() {
      writeln!(f, " - {}: {}", path, source)?;
    }
    writeln!(f, "sessions:")?;
    for (id, session) in self.sessions.iter() {
      writeln!(f, " - {}: {}", id, session)?;
    }
    Ok(())
  }

}

pub enum RegisterSessionError {
  NotFound,
  AlreadyExists,
}