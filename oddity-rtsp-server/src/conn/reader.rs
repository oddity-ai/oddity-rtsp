use std::net::TcpStream;

use oddity_rtsp_protocol::{
  RtspRequestReader,
  ResponseMaybeInterleaved,
  Error as RtspError,
};

use super::{
  conn::{
    MediaController,
    WriterTx,
  },
  handle::handle_request,
};

pub fn reader_loop(
  reader: RtspRequestReader<TcpStream>,
  media: MediaController,
  writer_tx: WriterTx,
) {
  loop {
    match reader.read() {
      Ok(request) => {
        match handle_request(
          &request,
          media.clone(),
        ) {
          Ok(response) => {
            if let Err(_) = writer_tx.send(
              ResponseMaybeInterleaved::Message(response)
            ) {
              tracing::error!("writer channel failed unexpectedly");
              break;
            }
          },
          Err(err) => {
            tracing::error!(
              %err, %request,
              "failed to handle request"
            );
          }
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
