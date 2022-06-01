use std::future::Future;

use oddity_rtsp_protocol::Request;

use crate::net::connection::ResponseSenderTx;

pub trait Handler {
  type Output: Future<Output = ()>;

  fn handle(request: Request, responder: ResponseSenderTx) -> Self::Output;

}