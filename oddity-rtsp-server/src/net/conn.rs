use std::net::{TcpStream, Shutdown};

use oddity_rtsp_protocol::{
  RtspRequestReader,
  RtspResponseWriter,
  ResponseMaybeInterleaved,
};

use crate::media::SharedMediaController;

use super::{
  reader,
  writer,
};

pub struct Connection {
  shutdown_handle: net::ShutdownHandle,
  reader: RtspRequestReader<TcpStream>,
  writer: RtspResponseWriter<TcpStream>,
  media: SharedMediaController,
  stop_rx: StopRx,
}

impl Connection {

  pub fn new(
    socket: TcpStream,
    media: &SharedMediaController,
    stop_rx: StopRx,
  ) -> Self {
    let (reader, writer, shutdown_handle) = net::split(socket);
    Self {
      shutdown_handle,
      reader,
      writer,
      media: media.clone(),
      stop_rx,
    }
  }

  pub fn run(
    mut self,
  ) {
    let (writer_tx, writer_rx) =
      channel::default::<ResponseMaybeInterleaved>();

    // TODO `Connection` does not self cleanup because it only stops
    // waiting for `stop_rx` even though it should also react to the
    // socket dying...

    let _reader_service = Service::spawn({
      let reader = self.reader;
      let media = self.media.clone();
      let writer_tx = writer_tx.clone();
      // Note: Don't need to use `_stop_rx` since we're using the
      // socket shutdown handle to signal cancellation to the I/O
      // reader and writer services.
      move |_stop_rx| reader::run_loop(
        reader,
        media,
        writer_tx,
      )
    });
    
    let _writer_service = Service::spawn({
      let writer = self.writer;
      move |stop_rx| writer::run_loop(
        writer,
        writer_rx,
        stop_rx,
      )
    });
    
    self.stop_rx.wait();
    if let Err(_) = self.shutdown_handle.shutdown(Shutdown::Both) {
      tracing::warn!("failed to shutdown socket");
    }
    
    tracing::trace!("connection stopping");
    // Dropping reader and writer services will automatically join.
  }
    
}