use std::thread;

use tokio::task;
use tokio::sync::mpsc;

use oddity_video as video;

use crate::media::{MediaDescriptor, MediaInfo};

type Result<T> = std::result::Result<T, video::Error>;

pub struct StreamReader {
  pub info: MediaInfo,
  handle: Option<thread::JoinHandle<()>>,
  packet_rx: mpsc::UnboundedReceiver<Result<video::Packet>>,
  stop_tx: mpsc::UnboundedSender<()>,
}

impl StreamReader {

  pub async fn new(
    descriptor: &MediaDescriptor,
  ) -> Result<Self> {
    let is_file = if let MediaDescriptor::File(_) = &descriptor { true } else { false };

    tracing::trace!(%descriptor, "initializing reader");
    let inner = backend::make_reader_with_sane_settings(descriptor.clone().into()).await?;
    tracing::trace!(%descriptor, "initialized reader");

    let info = MediaInfo::from_reader_best_video_stream(&inner)?;
    let stream_index = info.streams[0].index;
    tracing::trace!(%descriptor, stream_index=stream_index, "selected video stream");

    let (packet_tx, packet_rx) = mpsc::unbounded_channel();
    let (stop_tx, stop_rx) = mpsc::unbounded_channel();

    tracing::trace!(%descriptor, "starting stream reader");
    let handle = thread::spawn(
      move || Self::run(
        inner,
        stream_index,
        packet_tx,
        stop_rx,
        is_file,
      )
    );
    tracing::trace!(%descriptor, "started stream reader");

    Ok(Self {
      handle: Some(handle),
      info,
      packet_rx,
      stop_tx,
    })
  }

  pub async fn read(&mut self) -> Option<Result<video::Packet>> {
    self.packet_rx.recv().await
  }

  pub async fn stop(&mut self) {
    if let Ok(()) = self.stop_tx.send(()) {
      if let Some(handle) = self.handle.take() {
        tracing::trace!("sending stop signal to stream reader");
        let _ = task::spawn_blocking(|| handle.join()).await;
        tracing::trace!("stopped stream reader");
      }
    }
  }

  fn run(
    mut reader: video::Reader,
    stream_index: usize,
    packet_tx: mpsc::UnboundedSender<Result<video::Packet>>,
    mut stop_rx: mpsc::UnboundedReceiver<()>,
    is_file: bool,
  ) {
    loop {
      match stop_rx.try_recv() {
        Ok(()) |
        Err(mpsc::error::TryRecvError::Disconnected) => {
          tracing::trace!("stopping stream reader");
          break;
        },
        Err(mpsc::error::TryRecvError::Empty) => {}
      };

      let read = reader.read(stream_index);

      if is_file {
        // To pretend the file is a live stream, we need to wait a bit after
        // each packet or we'll overload the consumer.
        if let Ok(packet) = read.as_ref() {
          thread::sleep(packet.duration());
        }
      }

      let packet = match read {
        // Forward OK packets.
        Ok(packet) => {
          Some(Ok(packet))
        },
        // If the error was caused by an exhausted stream, try and see if we
        // can seek to the beginning of the file and then just keep reading:
        // we don't send a packet and just continue the loop in that case. If
        // seeking fails, forward the error.
        Err(video::Error::ReadExhausted) => {
          tracing::trace!("seeking to beginning of file after stream exhausted");
          match reader.seek(0) {
            Ok(()) => {
              // TODO! seeking will cause DTS to reset, but we want it to increase
              // monotically at all times
              None
            }
            Err(err) => {
              tracing::error!(%err, "failed to seek to beginning of file");
              Some(Err(err))
            }
          }
        },
        // Forward any errors.
        Err(err) => {
          Some(Err(err))
        },
      };

      if let Some(packet) = packet {
        if let Err(_) = packet_tx.send(packet) {
          tracing::trace!("packet channel broke");
          break;
        }
      }
    }
  }

}

impl Drop for StreamReader {

  fn drop(&mut self) {
    if self.handle.is_some() {
      panic!("Dropped `StreamReader` whilst running.");
    }
  }

}

// Holds functions that deal with the video backend stuff in `oddity_video`.
pub mod backend {

  use tokio::task;

  use oddity_video::{
    Reader,
    Options,
    Locator,
    Error,
  };

  pub async fn make_reader_with_sane_settings(
    locator: Locator,
  ) -> Result<Reader, Error> {
    task::spawn_blocking(move || {
        let options = match locator {
          Locator::Path(_) => {
            Default::default()
          },
          Locator::Url(_) => {
            // For streaming sources (live sources), we want to use TCP transport
            // over UDP and have sane timeouts.
            Options::new_with_rtsp_transport_tcp_and_sane_timeouts()
          }
        };

        Reader::new_with_options(
          &locator,
          &options,
        )
      })
      .await
      .unwrap()
  }

}