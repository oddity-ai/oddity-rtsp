use std::net::{
  ToSocketAddrs,
  TcpListener,
};
use std::error::Error;
use std::sync::{Arc, Mutex};

use concurrency::ServicePool;

use crate::media::{
  MediaController,
  SharedMediaController,
};

use super::conn::Connection;

pub struct Server<A: ToSocketAddrs + 'static> {
  addrs: A,
  media: SharedMediaController,
  connections: ServicePool,
}

impl<A: ToSocketAddrs + 'static> Server<A> {

  pub fn new(
    addrs: A,
    media: MediaController,
  ) -> Self {
    Self {
      addrs,
      media: Arc::new(
        Mutex::new(
          media
        )
      ),
      connections: ServicePool::new(),
    }
  }

  pub fn run(
    mut self
  ) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&self.addrs)?;
    loop {
      let (socket, addr) = listener.accept()?;
      tracing::trace!(%addr, "accepted client");

      self.connections.spawn({
        let media = self.media.clone();
        move |stop_rx| {
          Connection::new(
              socket,
              &media,
              stop_rx,
            )
            .run();
        }
      });
    }
  }

}