use std::error::Error;

use futures::StreamExt;

use tokio::net::{
  TcpListener,
  ToSocketAddrs,
};
use tokio_util::codec::Decoder;

use oddity_rtsp_protocol::{
  Request,
  Codec,
  AsServer,
};

pub struct Server<A: ToSocketAddrs> {
  addrs: A,
}

impl<A: ToSocketAddrs> Server<A> {

  pub fn new(addrs: A) -> Self {
    Self {
      addrs,
    }
  }

  pub async fn run(
    &self
  ) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&self.addrs).await?;

    loop {
      let (socket, addr) = listener.accept().await?;
      tracing::trace!("accepted client {}", addr);

      tokio::spawn(async move {
        let mut framed = Codec::<AsServer>::new().framed(socket);
        while let Some(Ok(request)) = framed.next().await {
          if let Err(err) = Self::handle(&request).await {
            tracing::error!("error handling request: {}", err);
          }
        }
      }).await?;
    }
  }

  pub async fn handle(
    request: &Request,
  ) -> Result<(), Box<dyn Error>> {
    /*match request {

    }*/
    Ok(())
  }

}