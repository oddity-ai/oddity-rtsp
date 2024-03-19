mod transport;

pub mod session_manager;
pub mod setup;

use std::error;
use std::fmt;

use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

use rand::Rng;

use oddity_rtsp_protocol as rtsp;
use video_rs as video;

use crate::media;
use crate::media::video::rtp_muxer;
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;
use crate::session::setup::{SessionSetup, SessionSetupTarget};
use crate::source::SourceDelegate;

pub enum SessionState {
    Stopped(SessionId),
}

pub type SessionStateTx = mpsc::UnboundedSender<SessionState>;
pub type SessionStateRx = mpsc::UnboundedReceiver<SessionState>;

pub type SessionStreamStateTx = broadcast::Sender<media::StreamState>;

pub enum SessionControlMessage {
    Play,
    StreamState,
}

pub type SessionControlTx = mpsc::UnboundedSender<SessionControlMessage>;
pub type SessionControlRx = mpsc::UnboundedReceiver<SessionControlMessage>;

pub struct Session {
    worker: Task,
    control_tx: SessionControlTx,
    stream_state_tx: SessionStreamStateTx,
}

impl Session {
    /// Any more than 16 media/stream info messages on the queue probably means
    /// something is really wrong and the server is overloaded.
    const MAX_QUEUED_INFO: usize = 16;

    pub async fn setup_and_start(
        id: SessionId,
        source_delegate: SourceDelegate,
        setup: SessionSetup,
        state_tx: SessionStateTx,
        runtime: &Runtime,
    ) -> Self {
        let (control_tx, control_rx) = mpsc::unbounded_channel();
        let (stream_state_tx, _) = broadcast::channel(Self::MAX_QUEUED_INFO);

        tracing::trace!(%id, "starting session");
        let worker = runtime
            .task()
            .spawn({
                let id = id.clone();
                let stream_state_tx = stream_state_tx.clone();
                |task_context| {
                    Self::run(
                        id,
                        source_delegate,
                        setup,
                        control_rx,
                        state_tx,
                        stream_state_tx,
                        task_context,
                    )
                }
            })
            .await;
        tracing::trace!(%id, "started session");

        Self {
            worker,
            control_tx,
            stream_state_tx,
        }
    }

    pub async fn play(
        &mut self,
        range: Option<rtsp::Range>,
    ) -> Result<media::StreamState, PlaySessionError> {
        if let Some(range) = range.as_ref() {
            tracing::trace!(%range, "checking if provided range is valid and supported");
            if !Self::is_range_supported(range) {
                tracing::error!(%range, "session does not support playing with this range");
                return Err(PlaySessionError::RangeNotSupported);
            }
        }

        let mut stream_state_rx = self.stream_state_tx.subscribe();
        tracing::trace!("querying session for stream state");
        self.control_tx
            .send(SessionControlMessage::StreamState)
            .map_err(|_| PlaySessionError::ControlBroken)?;

        let stream_state = stream_state_rx
            .recv()
            .await
            .map_err(|_| PlaySessionError::ControlBroken)?;
        tracing::trace!("received stream state");

        tracing::trace!("sending play signal to session");
        self.control_tx
            .send(SessionControlMessage::Play)
            .map_err(|_| PlaySessionError::ControlBroken)?;
        tracing::trace!("session playing");

        Ok(stream_state)
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
        control_rx: SessionControlRx,
        state_tx: SessionStateTx,
        stream_state_tx: SessionStreamStateTx,
        task_context: TaskContext,
    ) {
        let muxer = setup.rtp_muxer;

        match setup.rtp_target {
            SessionSetupTarget::RtpUdp(_) => {
                tracing::error!(%id, "started session with unsupported transport");
            }
            SessionSetupTarget::RtpTcp(target) => {
                tracing::trace!(%id, "starting rtp over tcp (interleaved) loop");
                Self::run_tcp_interleaved(
                    id.clone(),
                    source_delegate,
                    muxer,
                    target,
                    control_rx,
                    stream_state_tx,
                    task_context,
                )
                .await;
            }
        };

        let _ = state_tx.send(SessionState::Stopped(id));
    }

    async fn run_tcp_interleaved(
        id: SessionId,
        source_delegate: SourceDelegate,
        mut muxer: video::rtp::RtpMuxer,
        target: setup::SendInterleaved,
        mut control_rx: SessionControlRx,
        stream_state_tx: SessionStreamStateTx,
        mut task_context: TaskContext,
    ) {
        let mut state = SessionMediaState::Ready;
        let mut need_stream_state = false;

        let (mut source_reset_rx, mut source_packet_rx) = source_delegate.into_parts();

        'main: loop {
            select! {
                // CANCEL SAFETY: `broadcast::Receiver::recv` is cancel safe.
                reset = source_reset_rx.recv() => {
                    // If the source reader had an error and reinitialized its reader, then regained
                    // the connection, we must reinitialize our muxer as well to cope.
                    match reset {
                        Ok(media_info) => {
                            tracing::trace!("reinitializing muxer");
                            let new_muxer = rtp_muxer::make_rtp_muxer_builder()
                                .await
                                .and_then(|mut rtp_muxer_builder| {
                                    for stream_info in media_info.streams {
                                        tracing::trace!(
                                            stream_index=stream_info.index,
                                            "reinitializing muxer: adding stream to muxer",
                                        );
                                        rtp_muxer_builder = rtp_muxer_builder.with_stream(stream_info)?;
                                    }
                                    Ok(rtp_muxer_builder)
                                })
                                .map(|rtp_muxer_builder| rtp_muxer_builder.build());

                            match new_muxer {
                                Ok(new_muxer) => {
                                    muxer = new_muxer;
                                },
                                Err(err) => {
                                    tracing::error!(%err, %id, "failed to reinitialize muxer");
                                },
                            };
                        },
                        Err(_) => {
                            tracing::error!(%id, "source broken");
                            break;
                        },
                    }
                },
                // CANCEL SAFETY: `broadcast::Receiver::recv` is cancel safe.
                packet = source_packet_rx.recv() => {
                    match packet {
                        Ok(packet) => {
                            let (muxed, packet) = rtp_muxer::muxed(muxer, packet).await;
                            muxer = muxed;

                            if need_stream_state {
                                tracing::trace!(%id, "fetching stream state");
                                let (rtp_seq, rtp_timestamp) = muxer.seq_and_timestamp();
                                let stream_state = media::StreamState {
                                    rtp_seq,
                                    rtp_timestamp,
                                };
                                tracing::trace!(%id, rtp_seq, rtp_timestamp, "fetched stream state");
                                let _ = stream_state_tx.send(stream_state);

                                need_stream_state = false;
                            }

                            let packet = match packet {
                                Ok(packet) => packet,
                                Err(err) => {
                                    tracing::error!(%id, %err, "failed to mux packet");
                                    break;
                                }
                            };

                            if state == SessionMediaState::Playing {
                                let messages = packet.into_iter().map(|item| match item {
                                    video::rtp::RtpBuf::Rtp(payload) => {
                                        rtsp::ResponseMaybeInterleaved::Interleaved {
                                            channel: target.rtp_channel,
                                            payload: payload.into(),
                                        }
                                    }
                                    video::rtp::RtpBuf::Rtcp(payload) => {
                                        rtsp::ResponseMaybeInterleaved::Interleaved {
                                            channel: target.rtcp_channel,
                                            payload: payload.into(),
                                        }
                                    }
                                });

                                for message in messages {
                                    if let Err(err) = target.sender.send(message) {
                                        tracing::trace!(%id, %err, "underlying connection closed");
                                        break 'main;
                                    }
                                }
                            }
                        }
                        Err(_) => {
                            tracing::error!(%id, "source broken");
                            break;
                        }
                    }
                },
                // CANCEL SAFETY: `mpsc::UnboundedReceiver::recv` is cancel safe.
                message = control_rx.recv() => {
                    match message {
                        Some(SessionControlMessage::Play) => {
                            state = SessionMediaState::Playing;
                            tracing::info!(%id, "session now playing");
                        },
                        Some(SessionControlMessage::StreamState) => {
                            need_stream_state = true;
                            tracing::trace!(%id, "set need stream state flag");
                        },
                        None => {
                            tracing::error!(%id, "session control channel broke unexpectedly");
                            break;
                        },
                    };
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

    fn is_range_supported(range: &rtsp::Range) -> bool {
        match (range.start.as_ref(), range.end.as_ref()) {
            (Some(rtsp::NptTime::Now), None) => true,
            (Some(rtsp::NptTime::Time(start)), None) if *start <= 0.0 => true,
            _ => false,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    const SESSION_ID_LEN: u32 = 8;

    pub fn generate() -> SessionId {
        SessionId(
            rand::thread_rng()
                .sample(rand::distributions::Uniform::from(
                    10_u32.pow(Self::SESSION_ID_LEN - 1)..10_u32.pow(Self::SESSION_ID_LEN),
                ))
                .to_string(),
        )
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<&str> for SessionId {
    fn from(session_id: &str) -> Self {
        SessionId(session_id.to_string())
    }
}

#[derive(Debug)]
pub enum PlaySessionError {
    RangeNotSupported,
    ControlBroken,
}

impl fmt::Display for PlaySessionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PlaySessionError::RangeNotSupported => write!(f, "range not supported"),
            PlaySessionError::ControlBroken => write!(f, "failed to control session"),
        }
    }
}

impl error::Error for PlaySessionError {}

#[derive(PartialEq)]
enum SessionMediaState {
    Ready,
    Playing,
}
