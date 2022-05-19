use std::net::TcpStream;

use oddity_rtsp_protocol::RtspResponseWriter;
use concurrency::{

  channel,
  StopRx,
};

use super::conn::WriterRx;

pub fn writer_loop(
  writer: RtspResponseWriter<TcpStream>,
  writer_rx: WriterRx,
  stop_rx: StopRx,
) {
  loop {
    channel::select! {
      recv(writer_rx) -> response => {
        if let Ok(response) = response {
          if let Err(err) = writer.write(response) {
            tracing::error!(%err, "write failed");
            break;
          }
        } else {
          tracing::error!("writer channel failed unexpectedly");
          break;
        }
      },
      recv(stop_rx.into_rx()) -> _ => {
        tracing::trace!("connection writer stopping");
        break;
      },
    };
  }
}