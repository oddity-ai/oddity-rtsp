use tokio::select;
use tokio::sync::mpsc;
use tokio::sync::broadcast;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::media;

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
  Stop,
}

pub type SourceControlTx = mpsc::UnboundedSender<SourceControlMessage>;
pub type SourceControlRx = mpsc::UnboundedReceiver<SourceControlMessage>;

pub struct Source {
  control_tx: SourceControlTx,
  media_info_tx: SourceMediaInfoTx,
  packet_tx: SourcePacketTx,
  worker: Task,
}

impl Source {

  pub async fn start(
    path: SourcePath,
    state_tx: SourceStateTx,
    runtime: &Runtime,
  ) -> Self {
    let (control_tx, control_rx) = mpsc::unbounded_channel();
    let (media_info_tx, _) = broadcast::channel(16); // TODO magic constant
    let (packet_tx, _) = broadcast::channel(1024); // TODO magic constant

    let worker = runtime
      .task()
      .spawn({
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

    Self {
      control_tx,
      media_info_tx,
      packet_tx,
      worker,
    }
  }

  pub async fn stop(&mut self) {
    let _ = self.control_tx.send(SourceControlMessage::Stop);
    self.worker.stop().await;
  }

  // TODO STOPPING

  pub fn control_tx(&self) -> SourceControlTx {
    self.control_tx.clone()
  }

  pub fn subscribe_to_media_info(&self) -> SourceMediaInfoRx {
    self.media_info_tx.subscribe()
  }

  pub fn subscribe_to_packets(&self) -> SourcePacketRx {
    self.packet_tx.subscribe()
  }

  async fn run(
    path: SourcePath,
    mut control_rx: SourceControlRx,
    state_tx: SourceStateTx,
    media_info_tx: SourceMediaInfoTx,
    packet_tx: SourcePacketTx,
    mut task_context: TaskContext,
  ) {
    // TODO if the connection_sender_tx (inside Transport) dies the it is
    // similar to transport being closed (underlying connection died)
    loop {
      select! {
        _ = control_rx.recv() => {
          break;
        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      }
    }

    let _ = state_tx.send(SourceState::Stopped(path));
  }

}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SourcePath(String);