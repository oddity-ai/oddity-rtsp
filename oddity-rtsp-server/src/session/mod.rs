mod transport;

pub mod session_manager;
pub mod setup;

use std::fmt;

use tokio::select;
use tokio::sync::mpsc;

use rand::Rng;

use oddity_rtsp_protocol as rtsp;
use oddity_video as video;

use crate::runtime::Runtime;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::source::SourceDelegate;
use crate::session::setup::{SessionSetup, SessionSetupTarget};
use crate::media::video::rtp_muxer;

pub enum SessionState {
  Stopped(SessionId),
}

pub type SessionStateTx = mpsc::UnboundedSender<SessionState>;
pub type SessionStateRx = mpsc::UnboundedReceiver<SessionState>;

pub struct Session {
  worker: Task,
}

impl Session {

  pub async fn setup_and_start(
    id: SessionId,
    source_delegate: SourceDelegate,
    setup: SessionSetup,
    state_tx: SessionStateTx,
    runtime: &Runtime,
  ) -> Self {
    tracing::trace!(%id, "starting session");
    let worker = runtime
      .task()
      .spawn({
        let id = id.clone();
        |task_context| {
          Self::run(
            id,
            source_delegate,
            setup,
            state_tx,
            task_context,
          )
        }
      })
      .await;
    tracing::trace!(%id, "started session");

    Self {
      worker,
    }
  }

  pub async fn teardown(&mut self) {
    tracing::trace!("sending teardown signal to session");
    let _ = self.worker.stop().await;
    tracing::trace!("session torn down");
  }

  async fn run(
    id: SessionId,
    source_delegate: SourceDelegate,
    setup: SessionSetup,
    state_tx: SessionStateTx,
    task_context: TaskContext,
  ) {
    let muxer = setup.rtp_muxer;

    match setup.rtp_target {
      SessionSetupTarget::RtpUdp(_) => {
        tracing::error!(%id, "started session with unsupported transport");
      },
      SessionSetupTarget::RtpTcp(target) => {
        tracing::trace!(%id, "starting rtp over tcp (interleaved) loop");
        Self::run_tcp_interleaved(
          id.clone(),
          source_delegate,
          muxer,
          target,
          task_context,
        ).await;
      },
    };

    let _ = state_tx.send(SessionState::Stopped(id));
  }

  async fn run_tcp_interleaved(
    id: SessionId,
    mut source_delegate: SourceDelegate,
    mut muxer: video::RtpMuxer,
    target: setup::SendInterleaved,
    mut task_context: TaskContext,
  ) {
    loop {
      select! {
        // CANCEL SAFETY: `recv_packet` uses `broadcast::Receiver::recv` internally
        // which is cancel safe.
        packet = source_delegate.recv_packet() => {
          match packet {
            Some(packet) => {
              let (muxed, packet) = rtp_muxer::muxed(muxer, packet).await;
              muxer = muxed;

              let packet = match packet {
                Ok(packet) => packet,
                Err(err) => {
                  tracing::error!(%id, %err, "failed to mux packet");
                  break;
                },
              };

              let rtsp_interleaved_message = match packet {
                video::RtpBuf::Rtp(payload) => {
                  rtsp::ResponseMaybeInterleaved::Interleaved {
                    channel: target.rtp_channel,
                    payload: payload.into(),
                  }
                },
                video::RtpBuf::Rtcp(payload) => {
                  rtsp::ResponseMaybeInterleaved::Interleaved {
                    channel: target.rtcp_channel,
                    payload: payload.into(),
                  }
                },
              };

              if let Err(err) = target.sender.send(rtsp_interleaved_message) {
                tracing::trace!(%id, %err, "underlying connection closed");
                break;
              }
            }
            None => {
              tracing::error!(%id, "source broken");
              break;
            },
          }
        },
        // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
        _ = task_context.wait_for_stop() => {
          tracing::trace!("tearing down session");
          break;
        },
      }
    }

    tracing::trace!(%id, "finishing muxer");
    // Throw away possible last RTP buffer (we don't care about
    // it since this is real-time and there's no "trailer".
    let _ = rtp_muxer::finish(muxer).await;
    tracing::trace!(%id, "finished muxer");
  }

}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
  const SESSION_ID_LEN: usize = 16;

  pub fn generate() -> SessionId {
    SessionId(
      rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(Self::SESSION_ID_LEN)
        .map(char::from)
        .collect()
    )
  }

}

impl fmt::Display for SessionId {

  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    self.0.fmt(f)
  }

}