use std::sync::{Arc, Mutex};
use std::net::{TcpStream, Shutdown};

use oddity_rtsp_protocol::{
  RtspRequestReader,
  RtspResponseWriter,
  ResponseMaybeInterleaved,
};

use concurrency::{
  Service,
  StopRx,
  net,
  channel,
};

use crate::media;

use super::{
  reader::reader_loop,
  writer::writer_loop,
};

pub type MediaController = Arc<Mutex<media::Controller>>; // TODO refactor not the right place

pub type WriterRx = channel::Receiver<ResponseMaybeInterleaved>;
pub type WriterTx = channel::Sender<ResponseMaybeInterleaved>;

pub struct Connection {
  shutdown_handle: net::ShutdownHandle,
  reader: RtspRequestReader<TcpStream>,
  writer: RtspResponseWriter<TcpStream>,
  media: MediaController,
  stop_rx: StopRx,
}

impl Connection {

  pub fn new(
    socket: TcpStream,
    media: &MediaController,
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

    let reader_service = Service::spawn({
      let reader = self.reader;
      let media = self.media.clone();
      let writer_tx = writer_tx.clone();
      // Note: Don't need to use `_stop_rx` since we're using the
      // socket shutdown handle to signal cancellation to the I/O
      // reader and writer services.
      move |_stop_rx| reader_loop(
        reader,
        media,
        writer_tx,
      )
    });
    
    let writer_service = Service::spawn({
      let writer = self.writer;
      move |stop_rx| writer_loop(
        writer,
        writer_rx,
        stop_rx,
      )
    });
    
    self.stop_rx.wait();
    if let Err(_) = self.shutdown_handle.shutdown(Shutdown::Both) {
      tracing::warn!("failed to shutdown socket");
    }
    
    // Dropping reader and writer services will automatically join.
  }
    
}
