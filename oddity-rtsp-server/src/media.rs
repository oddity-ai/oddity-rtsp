mod stream;
mod multiplex;
mod file;

use std::sync::{Arc, Mutex};
use std::collections::{HashMap, hash_map::Entry};
use std::path::PathBuf;
use std::fmt;

use rand::Rng;

use oddity_rtsp_protocol::Uri;

use multiplex::{Multiplexer, MultiplexerService};

pub enum MediaDescriptor {
  Multiplexer {
    url: Uri,
  },
  Stream {
    url: Uri,
  },
  FileLoop {
    file: PathBuf,
  },
}

impl fmt::Display for MediaDescriptor {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MediaDescriptor::Multiplexer { url } =>
        write!(f, "multiplexer: {}", url),
      MediaDescriptor::Stream { url } =>
        write!(f, "stream: {}", url),
      MediaDescriptor::FileLoop { file } =>
        write!(f, "file loop: {}", file.display()),
    }
  }

}

pub trait MediaPlayer: Send {

}

pub type MediaSession = Box<dyn MediaPlayer>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate() -> SessionId {
    SessionId(
      rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(Self::SESSION_ID_LEN)
        .map(char::from)
        .collect())
  }

}

impl fmt::Display for SessionId {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.0.fmt(f)
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
  multiplexer_service: Arc<MultiplexerService>,
  descriptors: HashMap<String, MediaDescriptor>,
  sessions: HashMap<SessionId, Mutex<MediaSession>>,
}

impl MediaController {

  pub fn new() -> Self {
    Self {
      multiplexer_service: Arc::new(MultiplexerService::new()),
      descriptors: Default::default(),
      sessions: Default::default(),
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
  ) -> Result<SessionId, RegisterSessionError> {
    if let Some(descriptor) = self.descriptors.get_mut(path) {
      let session_id = SessionId::generate();
      if let Entry::Vacant(entry) = self.sessions.entry(session_id.clone()) {
        let player = match descriptor {
          MediaDescriptor::Multiplexer { url } => {
            // TODO init correctly
            Box::new(Multiplexer::new(&self.multiplexer_service))
          }
          _ => unimplemented!(), // TODO
        };
        entry.insert(Mutex::new(player));
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