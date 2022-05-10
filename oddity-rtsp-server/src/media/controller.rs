use std::sync::Mutex;
use std::collections::{HashMap, hash_map::Entry};
use std::fmt;

use super::{
  Descriptor,
  Source,
  SessionId,
  Session,
};

pub struct Controller {
  sources: HashMap<String, Source>,
  sessions: HashMap<SessionId, Mutex<Session>>,
}

impl Controller {

  pub fn new() -> Self {
    Self {
      sources: HashMap::new(),
      sessions: HashMap::new(),
    }
  }

  pub fn register_source(
    &mut self,
    path: &str,
    descriptor: &Descriptor,
  ) {
    let path = path
      .strip_prefix("/")
      .unwrap_or(path);

    let source = Source::new(descriptor);
    let _ = self.sources.insert(path.to_string(), source);
  }

  pub fn query_sdp(
    &self,
    path: &str,
  ) -> Option<String> {
    self
      .sources
      .get(path)
      .and_then(|source| {
        match source.describe() {
          Ok(source) => Some(source),
          Err(err) => {
            tracing::error!(
              path, %err,
              "failed to query SDP for stream"
            );
            None
          }
        }
      })
  }

  pub fn register_session(
    &mut self,
    path: &str,
  ) -> Result<SessionId, RegisterSessionError> {
    if let Some(source) = self.sources.get_mut(path) {
      let session_id = SessionId::generate();
      if let Entry::Vacant(entry) = self.sessions.entry(session_id.clone()) {
        let session = Session::new(source);
        entry.insert(Mutex::new(session));
        Ok(session_id)
      } else {
        Err(RegisterSessionError::AlreadyExists)
      }
    } else {
      Err(RegisterSessionError::NotFound)
    }
  }

  // TODO Below methods should be in media!
  // Hide mutex locking unlocking in these subfunctions

  // TODO Implement
  // pub fn play(
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

impl fmt::Display for Controller {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "media controller with {} sources and {} active sessions",
      self.sources.len(),
      self.sessions.len())
  }

}

pub enum RegisterSessionError {
  NotFound,
  AlreadyExists,
}