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
      match handle_request(&request).await {
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

}

/*
General:
- https://www.ffmpeg.org/doxygen/2.8/rtspenc_8c_source.html
- https://github.com/oddity-ai/oddity-rtsp-server/blob/master/rtsp/server.c

How to open RTP muxer and specify the port:
- https://ffmpeg.org/doxygen/2.1/rtpproto_8c.html#a4c0a351ea40886cc7efd4c4483b9d6a1
*/

#[tracing::instrument]
async fn handle_request(
  request: &Request,
) -> Result<Response, Box<dyn Error + Send>> {
  // Check the Require header and make sure all requested options are
  // supported or return response with 551 Option Not Supported.
  if !is_request_require_supported(request) {
    tracing::warn!(
      %request,
      "client asked for feature that is not supported");
    return Ok(reply_option_not_supported(request));
  }

  Ok(
    match request.method {
      /* Stateless */
      Method::Options => {
        reply_to_options_with_supported_methods(request)
      },
      Method::Announce => {
        reply_method_not_supported(request)
      },
      Method::Describe => {
        if is_request_one_of_content_types_supported(request) {
          unimplemented!();
        } else {
          tracing::warn!(
            %request,
            "none of content types accepted by client are supported");
          reply_not_acceptable(request)
        }
      },
      Method::GetParameter => {
        reply_method_not_supported(request)
      },
      Method::SetParameter => {
        reply_method_not_supported(request)
      },
      /* Stateful */
      Method::Setup => {
        unimplemented!();
      },
      Method::Play => {
        unimplemented!();
      },
      Method::Pause => {
        reply_method_not_supported(request)
      },
      Method::Record => {
        reply_method_not_supported(request)
      },
      Method::Teardown => {
        unimplemented!();
      },
      /* Invalid */
      // Request with method REDIRECT can only be sent from server to
      // client, not the other way around.
      Method::Redirect => {
        tracing::warn!(
          %request,
          "client tried redirect in request to server; \
          does client think it is server?");
        reply_method_not_valid(request)
      },
    }
  )
}

#[inline]
fn is_request_require_supported(
  request: &Request
) -> bool {
  // We don't support any features at this point
  request.require().is_none()
}

#[inline]
fn is_request_one_of_content_types_supported(
  request: &Request,
) -> bool {
  request.accept().contains(&"application/sdp")
}

#[inline]
fn reply_to_options_with_supported_methods(
  request: &Request,
) -> Response {
  Response::ok()
    .with_cseq_of(request)
    .with_header(
      "Public",
      "OPTIONS, DESCRIBE, SETUP, PLAY, TEARDOWN")
    .build()
}

#[inline]
fn reply_option_not_supported(
  request: &Request,
) -> Response {
  Response::error(Status::OptionNotSupported)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_method_not_supported(
  request: &Request,
) -> Response {
  tracing::warn!(method = %request.method, "client sent unsupported request");
  Response::error(Status::MethodNotAllowed)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_method_not_valid(
  request: &Request,
) -> Response {
  Response::error(Status::MethodNotValidInThisState)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_not_acceptable(
  request: &Request,
) -> Response {
  Response::error(Status::NotAcceptable)
    .with_cseq_of(request)
    .build()
}