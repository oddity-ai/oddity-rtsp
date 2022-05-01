use std::error::Error;
use std::sync::Arc;

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

use super::media::Controller as MediaController;

pub struct Server<A: ToSocketAddrs + 'static> {
  addrs: A,
  media: Arc<MediaController>,
}

impl<A: ToSocketAddrs + 'static> Server<A> {

  pub fn new(
    addrs: A,
    media: &Arc<MediaController>,
  ) -> Self {
    Self {
      addrs,
      media: Arc::clone(media),
    }
  }

  pub async fn run(
    &self
  ) -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind(&self.addrs).await?;

    // TODO This whole multi-threaded thing is great and all, but *NOT*
    //   having guaranteed ordering of the handling of incoming requests
    //   can easily break some of the assumptions of the RTSP protocol.
    //   We might need to see if this is true in our case, and if the
    //   easier option is to simply handle requests sequentially.
    loop {
      let (socket, addr) = listener.accept().await?;
      tracing::trace!(%addr, "accepted client");
      tokio::spawn(
        Self::handle_client(socket, self.media.clone()));
    }
  }

  #[inline]
  async fn handle_client(
    socket: TcpStream,
    media: Arc<MediaController>,
  ) {
    let mut framed = Codec::<AsServer>::new().framed(socket);
    while let Some(Ok(request)) = framed.next().await {
      tracing::trace!(%request, "C->S");
      match handle_request(&request, media.clone()).await {
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
TODO

General:
- https://www.ffmpeg.org/doxygen/2.8/rtspenc_8c_source.html
- https://github.com/oddity-ai/oddity-rtsp-server/blob/master/rtsp/server.c

How to open RTP muxer and specify the port:
- https://ffmpeg.org/doxygen/2.1/rtpproto_8c.html#a4c0a351ea40886cc7efd4c4483b9d6a1
*/

#[tracing::instrument(skip(media))]
async fn handle_request(
  request: &Request,
  media: Arc<MediaController>,
) -> Result<Response, Box<dyn Error + Send>> {
  // Check the Require header and make sure all requested options are
  // supported or return response with 551 Option Not Supported.
  if !is_request_require_supported(request) {
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
          if let Some(media_sdp) = media.query_sdp(request.path()) {
            reply_to_describe_with_media_sdp(request, media_sdp.clone())
          } else {
            reply_not_found(request)
          }
        } else {
          tracing::warn!(
            %request,
            "none of content types accepted by client are supported, \
             server only supports `application/sdp`");
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
        if request.session().is_none() {
          // 1. If client passed Session ID and media item is already in playing state
          //    then the client is trying to change transport parameters. We must respond
          //    negatively with 455 Method Not Valid In This State.
          //
          //    (We can also choose to ignore aggregate control on SETUP by always returning
          //     459 Aggregate Operation Not Allowed if the client has set Session at all
          //     times.)
          //
          //    X
          //
          // 2. Register a session with a newly generated Session ID.
          //
          // 3. Parse permissable Transport header and generate a workable Transport header
          //    from our side. This requires setting up the stream most likely to generate
          //    correct RTP/RTCP client and server port tuples.
          unimplemented!();
        } else {
          // RFC specification allows negatively responding to SETUP request with Session
          // IDs by responding with 459 Aggregate Operation Not Allowed. By handling this
          // here we don't have to deal with clients trying to change transport parameters
          // on media items that are already playing.
          reply_aggregate_operation_not_allowed(request)
        }
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
  // We only support SDP
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
fn reply_to_describe_with_media_sdp(
  request: &Request,
  sdp_contents: String,
) -> Response {
  Response::ok()
    .with_cseq_of(request)
    .with_sdp(sdp_contents)
    .build()
}

#[inline]
fn reply_option_not_supported(
  request: &Request,
) -> Response {
  tracing::debug!(
    %request,
    "client asked for feature that is not supported");
  Response::error(Status::OptionNotSupported)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_method_not_supported(
  request: &Request,
) -> Response {
  tracing::warn!(
    %request,
    method = %request.method,
    "client sent unsupported request");
  Response::error(Status::MethodNotAllowed)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_method_not_valid(
  request: &Request,
) -> Response {
  tracing::warn!(
    %request,
    method = %request.method,
    "client tried server-only method in request to server; \
     does client think it is server?");
  Response::error(Status::MethodNotValidInThisState)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_not_acceptable(
  request: &Request,
) -> Response {
  tracing::debug!(
    %request,
    "server does not support a presentation format acceptable \
     by client");
  Response::error(Status::NotAcceptable)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_not_found(
  request: &Request,
) -> Response {
  tracing::debug!(
    %request,
    path = request.path(),
    "path not registered as media item");
  Response::error(Status::NotFound)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_aggregate_operation_not_allowed(
  request: &Request,
) -> Response {
  tracing::debug!(
    %request,
    "refusing to do aggregate request");
  Response::error(Status::AggregateOperationNotAllowed)
    .with_cseq_of(request)
    .build()
}