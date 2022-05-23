use std::net::TcpStream;

use oddity_rtsp_protocol::{
  RtspRequestReader,
  ResponseMaybeInterleaved,
  Error as RtspError,
};

use crate::media::SharedMediaController;

use super::{
  writer::WriterTx,
  handle::handle_request,
};

pub fn run_loop(
  mut reader: RtspRequestReader<TcpStream>,
  media: SharedMediaController,
  writer_tx: WriterTx,
) {
  loop {
    match reader.read() {
      Ok(request) => {
        let response = handle_request(
          &request,
          media.clone(),
          // TODO handle disconnection with teardown (?)
          writer_tx.clone(),
        );

        if let Err(_) = writer_tx.send(
          ResponseMaybeInterleaved::Message(response)
        ) {
          tracing::error!("writer channel failed unexpectedly");
          break;
        }
      },
      Err(RtspError::Shutdown) => {
        tracing::trace!("connection reader stopping");
        break;
      },
      Err(err) => {
        tracing::error!(%err, "read failed");
        break;
      },
    }
  }
}
