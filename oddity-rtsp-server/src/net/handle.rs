use oddity_rtsp_protocol::{
  Request,
  Response,
  Method,
  Status,
};

use crate::{
  net::WriterTx,
  media::{
    SharedMediaController,
    RegisterSessionError,
    session::{
      SessionId,
      make_session_context_from_transport,
    },
  },
};

/*
TODO

General:
- https://www.ffmpeg.org/doxygen/2.8/rtspenc_8c_source.html
- https://github.com/oddity-ai/oddity-rtsp-server/blob/master/rtsp/server.c

How to open RTP muxer and specify the port:
- https://ffmpeg.org/doxygen/2.1/rtpproto_8c.html#a4c0a351ea40886cc7efd4c4483b9d6a1
*/

#[tracing::instrument(skip(media))]
pub fn handle_request(
  request: &Request,
  media: SharedMediaController,
  writer_tx: WriterTx,
) -> Response {
  // Shorthand for unlocking the media controller.
  macro_rules! media {
    () => { media.lock().unwrap() };
  }

  // Check the Require header and make sure all requested options are
  // supported or return response with 551 Option Not Supported.
  if !is_request_require_supported(request) {
    return reply_option_not_supported(request);
  }

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
        if let Some(media_sdp) = media!().query_sdp(request.path()) {
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
      if request.session().is_some() {
        // RFC specification allows negatively responding to SETUP request with Session
        // IDs by responding with 459 Aggregate Operation Not Allowed. By handling this
        // here we don't have to deal with clients trying to change transport parameters
        // on media items that are already playing.
        return reply_aggregate_operation_not_allowed(request);
      }

      let transport = match request.transport() {
        Ok(transport) => transport,
        Err(_) => {
          // If the client did not provide a valid transport header value, then there
          // no way to reach it and we return "Unsupported Transport".
          return reply_unsupported_transport(request);
        }
      };

      let session_context = match make_session_context_from_transport(
        transport,
        writer_tx,
      ) {
        Ok(session_context) => session_context,
        Err(_) => {
          // If we cannot create a session from the given transport parameters, then this
          // usually means that the client provided an invalid or unsupported `Transport`
          // header parameterization.
          return reply_unsupported_transport(request);
        }
      };

      match media!().register_session(request.path(), session_context) {
        // Session was successfully registered!
        Ok(session_id) => {
          reply_to_setup_with_session_id(request, &session_id)
        },
        // Path not found, source does not exist.
        Err(RegisterSessionError::NotFound) => {
          reply_not_found(request)
        },
        // In the highly unlikely case that the randomly generated session was already
        // in use before.
        Err(RegisterSessionError::AlreadyExists) => {
          tracing::error!(
            %request,
            "session id already present (collision)");
          reply_internal_server_error(request)
        },
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
fn reply_to_setup_with_session_id(
  request: &Request,
  session_id: &SessionId,
) -> Response {
  Response::ok()
    .with_cseq_of(request)
    .with_header("Session", session_id)
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

#[inline]
fn reply_unsupported_transport(
  request: &Request,
) -> Response {
  tracing::debug!(
    %request,
    "unsupported transport");
  Response::error(Status::UnsupportedTransport)
    .with_cseq_of(request)
    .build()
}

#[inline]
fn reply_internal_server_error(
  request: &Request,
) -> Response {
  Response::error(Status::InternalServerError)
    .with_cseq_of(request)
    .build()
}