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
  Status,
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
      tracing::trace!(%addr, "accepted client");
      tokio::spawn(Self::handle_client(socket));
    }
  }

  #[inline]
  async fn handle_client(
    socket: TcpStream,
  ) {
    let mut framed = Codec::<AsServer>::new().framed(socket);
    while let Some(Ok(request)) = framed.next().await {
      tracing::trace!(%request, "C->S");
      match Self::handle_request(&request).await {
        Ok(response) => {
          tracing::trace!(%response, "S->C");
          if let Err(err) = framed.send(response).await {
            tracing::error!(%err, "error trying to send response");
          }
        },
        Err(err) => {
          tracing::error!(%err, "error handling request");
        },
      }
    }
  }

  #[tracing::instrument]
  async fn handle_request(
    request: &Request,
  ) -> Result<Response, Box<dyn Error + Send>> {
    Ok(match request.method {
      /* Stateless */
      Method::Options => {
        Response::ok()
          .with_cseq_of(request)
          .with_header("Public", "OPTIONS, DESCRIBE, SETUP, PLAY, TEARDOWN")
          .build()
      },
      Method::Announce => {
        tracing::warn!("client sent unsupported request: ANNOUNCE");
        Response::error(Status::MethodNotAllowed)
          .with_cseq_of(request)
          .build()
      },
      Method::Describe => {
        unimplemented!();
      },
      Method::GetParameter => {
        tracing::warn!("client sent unsupported request: GET_PARAMETER");
        Response::error(Status::MethodNotAllowed)
          .with_cseq_of(request)
          .build()
      },
      Method::SetParameter => {
        tracing::warn!("client sent unsupported request: SET_PARAMETER");
        Response::error(Status::MethodNotAllowed)
          .with_cseq_of(request)
          .build()
      },
      /* Stateful */
      Method::Setup => {
        unimplemented!();
      },
      Method::Play => {
        unimplemented!();
      },
      Method::Pause => {
        tracing::warn!("client sent unsupported request: PAUSE");
        Response::error(Status::MethodNotAllowed)
          .with_cseq_of(request)
          .build()
      },
      Method::Record => {
        tracing::warn!("client sent unsupported request: RECORD");
        Response::error(Status::MethodNotAllowed)
          .with_cseq_of(request)
          .build()
      },
      Method::Teardown => {
        unimplemented!();
      },
      /* Invalid */
      // Request with method REDIRECT can only be sent from Server->Client,
      // not the other way around.
      Method::Redirect => {
        tracing::warn!(
          %request,
          "client tried redirect in request to server; \
           does client think it is server?");
        Response::error(Status::MethodNotValidInThisState)
          .with_cseq_of(request)
          .build()
      },
    })
  }

}