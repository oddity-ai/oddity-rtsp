pub mod source_manager;

use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::broadcast;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::media::{self, MediaDescriptor};

pub enum SourceState {
  Stopped(SourcePath),
}

pub type SourceStateTx = mpsc::UnboundedSender<SourceState>;
pub type SourceStateRx = mpsc::UnboundedReceiver<SourceState>;

pub type SourceMediaInfoTx = broadcast::Sender<media::MediaInfo>;
pub type SourceMediaInfoRx = broadcast::Receiver<media::MediaInfo>;

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

  pub async fn start(
    name: &str,
    path: SourcePath,
    descriptor: MediaDescriptor,
    state_tx: SourceStateTx,
    runtime: &Runtime,
  ) -> Self {
    let (control_tx, control_rx) = mpsc::unbounded_channel();
    let (media_info_tx, _) = broadcast::channel(Self::MAX_QUEUED_MEDIA_INFO);
    let (packet_tx, _) = broadcast::channel(Self::MAX_QUEUED_PACKETS);

    tracing::trace!(name, %path, "starting source");
    let worker = runtime
      .task()
      .spawn({
        let path = path.clone();
        let media_info_tx = media_info_tx.clone();
        let packet_tx = packet_tx.clone();
        move |task_context| {
          Self::run(
            path,
            control_rx,
            state_tx,
            media_info_tx,
            packet_tx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!(name, %path, "started source");

    Self {
      name: name.to_string(),
      path,
      descriptor,
      control_tx,
      media_info_tx,
      packet_tx,
      worker,
    }
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
      packet_rx: self.packet_tx.subscribe(),
    }
  }

  async fn run(
    path: SourcePath,
    mut control_rx: SourceControlRx,
    state_tx: SourceStateTx,
    media_info_tx: SourceMediaInfoTx,
    packet_tx: SourcePacketTx,
    mut task_context: TaskContext,
  ) {
    // TODO! implement
    loop {
      select! {
        _ = control_rx.recv() => {
          break;
        },
        _ = task_context.wait_for_stop() => {
          tracing::trace!("stopping source");
          break;
        },
      }
    }

    let _ = state_tx.send(SourceState::Stopped(path));
  }

}

pub struct SourceDelegate {
  control_tx: SourceControlTx,
  media_info_rx: SourceMediaInfoRx,
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

  pub async fn recv_packet(&mut self) -> Option<media::Packet> {
    self.packet_rx.recv().await.ok()
  }

}

pub type SourcePath = String;
pub type SourcePathRef = str;