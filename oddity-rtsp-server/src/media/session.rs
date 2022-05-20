use std::fmt;

use rand::Rng;

use concurrency::{
  Service,
  StopRx,
};

use super::{
  Source,
  SourceRx,
};

pub struct Session {
  service: Option<Service>,
  source_rx: SourceRx,
}

impl Session {

  pub fn new(source: &mut Source) -> Self {
    let service = Service::spawn({
      let source_rx = source.subscribe();
      move |stop| {
        Self::run(
          source_rx,
          stop,
        )
      }
    });

    Self {
      service: Some(service),
      source_rx: source.subscribe(),
    }
  }

  pub fn play() {
    // TODO
  }

  // TODO drop() = teardown (?)
  pub fn teardown(self) {
    
  }

  fn run(
    // TODO target
    source_rx: SourceRx,
    stop: StopRx,
  ) {
    
  }

}

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