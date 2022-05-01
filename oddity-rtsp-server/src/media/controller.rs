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
  sources: HashMap<Descriptor, Source>,
  descriptors: HashMap<String, Descriptor>,
  sessions: HashMap<SessionId, Mutex<Session>>,
}

impl Controller {

  pub fn new() -> Self {
    Self {
      sources: HashMap::new(),
      descriptors: HashMap::new(),
      sessions: HashMap::new(),
    }
  }

  pub fn register_descriptor(
    &mut self,
    path: &str,
    descriptor: Descriptor,
  ) {
    let _ = self.descriptors.insert(path.to_string(), descriptor);
  }

  pub fn query_sdp(
    &self,
    path: &str,
  ) -> Option<String> {
    // TODO
    None
  }

  pub fn register_session(
    &mut self,
    path: &str,
  ) -> Result<SessionId, RegisterSessionError> {
    if let Some(descriptor) = self.descriptors.get_mut(path) {
      let session_id = SessionId::generate();
      if let Entry::Vacant(entry) = self.sessions.entry(session_id.clone()) {
        let session = Session::new();
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
    writeln!(f, "descriptors:")?;
    for (path, source) in self.descriptors.iter() {
      writeln!(f, " - {}: {}", path, source)?;
    }
    writeln!(f, "sessions:")?;
    //for (id, session) in self.sessions.iter() {
    //  writeln!(f, " - {}: {}", id, session)?;
    //}
    Ok(())
  }

}

pub enum RegisterSessionError {
  NotFound,
  AlreadyExists,
}