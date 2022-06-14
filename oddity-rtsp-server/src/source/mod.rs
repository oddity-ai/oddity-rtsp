pub mod source_manager;

use std::time;

use tokio::{select, pin};
use tokio::time::{sleep, timeout};
use tokio::sync::mpsc;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;

use oddity_video as video;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::media::{self, MediaDescriptor, MediaInfo};
use crate::media::video::reader;

pub enum SourceState {
  Stopped(SourcePath),
}

pub type SourceStateTx = mpsc::UnboundedSender<SourceState>;
pub type SourceStateRx = mpsc::UnboundedReceiver<SourceState>;

pub type SourceMediaInfoTx = broadcast::Sender<media::MediaInfo>;
pub type SourceMediaInfoRx = broadcast::Receiver<media::MediaInfo>;

pub type SourceResetTx = broadcast::Sender<media::MediaInfo>;
pub type SourceResetRx = broadcast::Receiver<media::MediaInfo>;

pub type SourcePacketTx = broadcast::Sender<media::Packet>;
pub type SourcePacketRx = broadcast::Receiver<media::Packet>;

pub enum SourceControlMessage {
  StreamInfo,
}

pub type SourceControlTx = mpsc::UnboundedSender<SourceControlMessage>;
pub type SourceControlRx = mpsc::UnboundedReceiver<SourceControlMessage>;

pub struct Source {
  pub name: String,
  pub path: SourcePath,
  pub descriptor: MediaDescriptor,
  control_tx: SourceControlTx,
  media_info_tx: SourceMediaInfoTx,
  reset_tx: SourceResetTx,
  packet_tx: SourcePacketTx,
  worker: Task,
}

impl Source {
  /// Any more than 16 stream info messages on the queue probably means
  /// something is really wrong and the server is overloaded.
  const MAX_QUEUED_MEDIA_INFO: usize = 16;

  /// Any more than 1024 packets queued probably indicates the server is
  /// terribly overloaded/broken.
  const MAX_QUEUED_PACKETS: usize = 1024;

  /// Number of seconds between retries.
  const RETRY_DELAY_SECS: u64 = 60;

  pub async fn start(
    name: &str,
    path: SourcePath,
    descriptor: MediaDescriptor,
    state_tx: SourceStateTx,
    runtime: &Runtime,
  ) -> Result<Self, video::Error> {
    let path = normalize_path(path);
    let (reader, media_info) = Self::initialize_stream(&descriptor).await?;

    let (control_tx, control_rx) = mpsc::unbounded_channel();
    let (media_info_tx, _) = broadcast::channel(Self::MAX_QUEUED_MEDIA_INFO);
    let (reset_tx, _) = broadcast::channel(Self::MAX_QUEUED_MEDIA_INFO);
    let (packet_tx, _) = broadcast::channel(Self::MAX_QUEUED_PACKETS);

    tracing::trace!(name, %path, "starting source");
    let worker = runtime
      .task()
      .spawn({
        let path = path.clone();
        let descriptor = descriptor.clone();
        let media_info_tx = media_info_tx.clone();
        let reset_tx = reset_tx.clone();
        let packet_tx = packet_tx.clone();
        move |task_context| {
          Self::run(
            path,
            descriptor,
            reader,
            media_info,
            control_rx,
            state_tx,
            media_info_tx,
            reset_tx,
            packet_tx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!(name, %path, "started source");

    Ok(Self {
      name: name.to_string(),
      path,
      descriptor,
      control_tx,
      media_info_tx,
      reset_tx,
      packet_tx,
      worker,
    })
  }

  pub async fn stop(&mut self) {
    tracing::trace!("sending stop signal to source");
    self.worker.stop().await;
    tracing::trace!("stopped source");
  }

  pub fn delegate(&mut self) -> SourceDelegate {
    SourceDelegate {
      control_tx: self.control_tx.clone(),
      media_info_rx: self.media_info_tx.subscribe(),
      reset_rx: self.reset_tx.subscribe(),
      packet_rx: self.packet_tx.subscribe(),
    }
  }

  async fn run(
    path: SourcePath,
    descriptor: MediaDescriptor,
    mut reader: video::Reader,
    mut media_info: MediaInfo,
    mut control_rx: SourceControlRx,
    state_tx: SourceStateTx,
    media_info_tx: SourceMediaInfoTx,
    reset_tx: SourceResetTx,
    packet_tx: SourcePacketTx,
    mut task_context: TaskContext,
  ) {
    let is_file = if let MediaDescriptor::File(_) = &descriptor { true } else { false };

    'outer: loop {
      let stream_index = match media_info.streams.first() {
        Some(stream) => {
          tracing::trace!(%path, stream_index=stream.index, "selected video stream");
          stream.index
        },
        None => {
          tracing::error!(%path, "recevied media info without stream");
          return;
        },
      };

      let reader_stream = reader::into_stream(reader, stream_index);
      pin!(reader_stream);

      'inner: loop {
        select! {
          // CANCEL SAFETY: `StreamExt:next` is always cancel safe.
          packet = reader_stream.next() => {
            match packet {
              Some(Ok(packet)) => {
                let _ = packet_tx.send(packet.clone());
                if is_file {
                  // To pretend the file is a live stream, we need to wait a bit after
                  // each packet or we'll overload the consumer.
                  sleep(packet.duration()).await;
                }
              },
              Some(Err(err)) => {
                tracing::error!(%path, %err, "failed to read video stream");
                break 'inner;
              },
              None => {
                tracing::info!(%path, "video stream ended");
                if is_file {
                  // TODO! instead of doing break here we should just access the reader
                  // and seek back to the beginning but this requires some refactoring...
                  break 'inner;
                } else {
                  break 'inner;
                }
              },
            };
          },
          // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
          message = control_rx.recv() => {
            match message {
              Some(SourceControlMessage::StreamInfo) => {
                let _ =  media_info_tx.send(media_info.clone());
              },
              None => {
                tracing::error!(%path, "source control channel broke unexpectedly");
                break 'outer;
              },
            };
          },
          // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
          _ = task_context.wait_for_stop() => {
            tracing::trace!(%path, "stopping source");
            break 'outer;
          },
        }
      }

      'restart: loop {
        match Self::initialize_stream(&descriptor).await {
          Ok((new_reader, new_media_info)) => {
            reader = new_reader;
            media_info = new_media_info;
            let _ = reset_tx.send(media_info.clone());
            
            break 'restart;
          },
          Err(err) => {
            tracing::error!(
              %err, %descriptor, retry_delay=Self::RETRY_DELAY_SECS,
              "failed to restart reader (waiting before retrying)",
            );
            // We want to wait some time before retrying. We wrap `wait_for_stop` in
            // a timeout to achieve this ...
            match timeout(
              time::Duration::from_secs(Self::RETRY_DELAY_SECS),
              task_context.wait_for_stop(),
            ).await {
              Ok(()) => {
                tracing::trace!(%path, "stopping source (during stream restart)");
                // If `wait_for_stop` returns, we break out of the outer loop and stop ...
                break 'outer;
              },
              Err(_) => {
                // But if the timeout is reached, we simply restart this loop to try and
                // see if we can get the reader to work this time.
                continue 'restart;
              },
            }
          },
        }
      }
    }

    let _ = state_tx.send(SourceState::Stopped(path));
  }

  async fn initialize_stream(
    descriptor: &MediaDescriptor,
  ) -> Result<(video::Reader, MediaInfo), video::Error> {
    tracing::trace!(%descriptor, "initializing stream");
    let reader = reader::make_reader_with_sane_settings(descriptor.clone().into()).await?;
    let media_info = MediaInfo::from_reader_best_video_stream(&reader)?;
    tracing::trace!(%descriptor, "initialized stream");
    Ok((reader, media_info))
  }

}

pub struct SourceDelegate {
  control_tx: SourceControlTx,
  media_info_rx: SourceMediaInfoRx,
  reset_rx: SourceResetRx,
  packet_rx: SourcePacketRx,
}

impl SourceDelegate {

  pub async fn query_media_info(&mut self) -> Option<media::MediaInfo> {
    if let Ok(()) = self.control_tx.send(SourceControlMessage::StreamInfo) {
      self.media_info_rx.recv().await.ok()
    } else {
      None
    }
  }

  pub fn into_parts(self) -> (SourceResetRx, SourcePacketRx) {
    (self.reset_rx, self.packet_rx)
  }

}

pub type SourcePath = String;
pub type SourcePathRef = str;

pub fn normalize_path(path: SourcePath) -> SourcePath {
  if path.starts_with("/") {
    path
  } else {
    format!("/{}", &path)
  }
}