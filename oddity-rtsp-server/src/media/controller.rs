use std::sync::Mutex;
use std::collections::{HashMap, hash_map::Entry};
use std::fmt;

use super::{
  Descriptor,
  source::Source,
  session::{
    Session,
    SessionId,
  },
};

pub struct Controller {
  sources: HashMap<String, Source>,
  // TODO mutex here should not be necessary anymore since there's one in server as well
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
    let _ = self.sources.insert(
      normalize_url_path(path).to_string(),
      Source::new(descriptor),
    );
  }

  pub fn query_sdp(
    &self,
    path: &str,
  ) -> Option<String> {
    // TODO we can do better
    self
      .sources
      .get(normalize_url_path(path))
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
    if let Some(source) = self.sources.get_mut(normalize_url_path(path)) {
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

fn normalize_url_path(path: &str) -> &str {
  path
    .strip_prefix("/")
    .unwrap_or(path)
}