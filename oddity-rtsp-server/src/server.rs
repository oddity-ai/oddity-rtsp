use std::error::Error;

use futures::{StreamExt, SinkExt};

use tokio::net::{
  TcpListener,
  TcpStream,
  ToSocketAddrs,
};
use tokio_util::codec::Decoder;

use oddity_rtsp_protocol::{
  Request,
  Response,
  Headers,
  Codec,
  AsServer,
  Method,
};

pub struct Server<A: ToSocketAddrs + 'static> {
  addrs: A,
}

impl<A: ToSocketAddrs + 'static> Server<A> {

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
      tracing::trace!("accepted client: {}", addr);
      tokio::spawn(Self::handle_client(socket));
    }
  }

  #[inline]
  async fn handle_client(
    socket: TcpStream,
  ) {
    let mut framed = Codec::<AsServer>::new().framed(socket);
    while let Some(Ok(request)) = framed.next().await {
      tracing::trace!("C->S: {}", &request);
      match Self::handle_request(&request).await {
        Ok(response) => {
          tracing::trace!("S->C: {}", &response);
          if let Err(err) = framed.send(response).await {
            tracing::error!("error trying to send response: {}", err);
          }
        },
        Err(err) => {
          tracing::error!("error handling request: {}", err);
        },
      }
    }
  }

  async fn handle_request(
    request: &Request,
  ) -> Result<Response, Box<dyn Error + Send>> {
    Ok(match request.method {
      /* Stateless */
      Method::Options => {
        Response::to(
          request,
          Headers::from([
            ("Public".to_string(), "OPTIONS, DESCRIBE, SETUP, PLAY, TEARDOWN".to_string())
          ]))
      },
      Method::Announce => {
        Response::error(405, "Method Not Allowed")
      },
      Method::Describe => {
        unimplemented!();
      },
      Method::GetParameter => {
        Response::error(405, "Method Not Allowed")
      },
      Method::SetParameter => {
        Response::error(405, "Method Not Allowed")
      },
      /* Stateful */
      Method::Setup => {
        unimplemented!();
      },
      Method::Play => {
        unimplemented!();
      },
      Method::Pause => {
        Response::error(405, "Method Not Allowed")
      },
      Method::Record => {
        Response::error(405, "Method Not Allowed")
      },
      Method::Teardown => {
        unimplemented!();
      },
      /* Invalid */
      // Request with method REDIRECT can only be sent from Server->Client,
      // not the other way around.
      Method::Redirect => {
        tracing::warn!(
          "client tried redirect in request to server; \
           does client think it is server?");
        Response::error(455, "Method Not Valid in This State")
      },
    })
  }

}