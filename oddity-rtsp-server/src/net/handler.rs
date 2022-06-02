use oddity_rtsp_protocol::Request;

use crate::net::connection::Writer;

pub trait Handler: Send + Sync + 'static {

  fn handle(&self, request: Request, writer: &Writer);

}