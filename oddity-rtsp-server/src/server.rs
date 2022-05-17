use std::error::Error;
// TODO should use tokio mutex
use std::sync::Arc;

use super::media;
use super::connection::Connection;

// TODO duplicate
type MediaController = Arc<Mutex<media::Controller>>;

pub struct Server<A: ToSocketAddrs + 'static> {
  addrs: A,
  media: MediaController,
}

impl<A: ToSocketAddrs + 'static> Server<A> {

  pub fn new(
    addrs: A,
    media: media::Controller,
  ) -> Self {
    Self {
      addrs,
      media: Arc::new(
        Mutex::new(
          media
        )
      ),
    }
  }

  pub async fn run(
    self
  ) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&self.addrs).await?;
    loop {
      let (socket, addr) = listener.accept().await?;
      tracing::trace!(%addr, "accepted client");
      tokio::spawn(Connection::spawn(socket, self.media.clone()));
    }
  }

}
