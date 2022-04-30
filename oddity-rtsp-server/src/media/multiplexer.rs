use std::sync::Arc;

use super::MediaPlayer;

pub struct MultiplexerService {

}

impl MultiplexerService {

  pub fn new() -> Self {
    MultiplexerService {
    }
  }

}

pub struct Multiplexer {
  service: Arc<MultiplexerService>,
}

impl Multiplexer {

  pub fn new(service: &Arc<MultiplexerService>) -> Self {
    Multiplexer {
      service: service.clone(),
    }
  }

}

impl MediaPlayer for Multiplexer {

}
