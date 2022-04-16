mod stream;
mod multiplex;

use std::fmt;
use std::rc::Rc;
use std::collections::{HashMap, hash_map::Entry};

use rand::Rng;

use oddity_rtsp_protocol::Uri;

pub enum MediaDescriptor {
  Multiplexer {
    url: Uri,
  },
  // TODO
}

impl fmt::Display for MediaDescriptor {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MediaDescriptor::Multiplexer { url } =>
        write!(f, "multiplexer: {}", url),
    }
  }

}

pub struct Session {
  state: State,
  manager: Rc<dyn MediaManager>,
}

impl Session {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate_id() -> String {
    rand::thread_rng()
      .sample_iter(&rand::distributions::Alphanumeric)
      .take(Self::SESSION_ID_LEN)
      .map(char::from)
      .collect()
  }

}

impl fmt::Display for Session {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    // TODO write!(f, "{} ({})", self.state, /* TODO */)
    Ok(())
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

pub struct MediaController {
  descriptors: HashMap<String, MediaDescriptor>,
  // TODO can we not get rid of this whole Rc dance by mapping
  // sessions to media descriptors, and resolving the descriptor
  // to the correct manager...???
  sessions: HashMap<String, Session>,
  // TODO stream_manager: Rc<StreamManager>,
  stream_multiplex_manager: Rc<MultiplexManager>, // TODO name?
}

impl MediaController {

  pub fn new() -> Self {
    Self {
      descriptors: HashMap::new(),
      sessions: HashMap::new(),
      stream_multiplex_manager: Rc::new(MultiplexManager {}),
    }
  }

  pub fn register_descriptor(
    &mut self,
    path: &str,
    descriptor: MediaDescriptor,
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
  ) -> Result<(String, &Session), RegisterSessionError> {
    if let Some(descriptor) = self.descriptors.get_mut(path) {
      let session_id = Session::generate_id();
      if let Entry::Vacant(entry) = self.sessions.entry(session_id) {
        match descriptor {
          MediaDescriptor::Multiplexer { url } => {
            // TODO new and formatting???
            Ok((
              session_id,
              entry.insert(Session {
                state: State::Init,
                manager: self.stream_multiplex_manager.clone()
              })
            ))
          },
        }
        // TODO entry.insert(Session { /* TODO */})

      } else {
        Err(RegisterSessionError::AlreadyExists)
      }
    } else {
      Err(RegisterSessionError::NotFound)
    }
  }

  pub fn session(
    &self,
    session_id: &str,
  ) -> Option<&Session> {
    // TODO
    None
  }

  // TODO Below methods should be in media!

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
    writeln!(f, "descriptors:")?;
    for (path, source) in self.descriptors.iter() {
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

pub trait MediaManager {

  fn play(&self, session: &Session);

}

pub struct MultiplexManager;

impl MediaManager for MultiplexManager {

  fn play(&self, session: &Session) {
    unimplemented!()
  }

}