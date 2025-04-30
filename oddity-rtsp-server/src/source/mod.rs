pub mod source_manager;

use std::sync::Arc;
use std::time;

use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::time::timeout;

use video_rs as video;

use crate::media::video::reader::StreamReader;
use crate::media::MediaInfo;
use crate::media::{self, MediaDescriptor};
use crate::runtime::task_manager::{Task, TaskContext};
use crate::runtime::Runtime;

pub enum SourceState {
    Stopped(SourcePath),
}

pub type SourceStateTx = mpsc::UnboundedSender<SourceState>;
pub type SourceStateRx = mpsc::UnboundedReceiver<SourceState>;

pub type SourceResetTx = broadcast::Sender<media::MediaInfo>;
pub type SourceResetRx = broadcast::Receiver<media::MediaInfo>;

pub type SourcePacketTx = broadcast::Sender<media::Packet>;
pub type SourcePacketRx = broadcast::Receiver<media::Packet>;

pub struct Source {
    pub name: String,
    // pub path: SourcePath,
    pub descriptor: MediaDescriptor,
    reset_tx: SourceResetTx,
    packet_tx: SourcePacketTx,
    media_info: Arc<Mutex<Option<MediaInfo>>>,
    worker: Task,
}

impl Source {
    /// Any more than 16 media/stream info messages on the queue probably means
    /// something is really wrong and the server is overloaded.
    const MAX_QUEUED_INFO: usize = 16;

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

        let (reset_tx, _) = broadcast::channel(Self::MAX_QUEUED_INFO);
        let (packet_tx, _) = broadcast::channel(Self::MAX_QUEUED_PACKETS);
        let media_info = Arc::new(Mutex::new(None));

        tracing::trace!(name, %path, "starting source");
        let worker = runtime
            .task()
            .spawn({
                let path = path.clone();
                let descriptor = descriptor.clone();
                let reset_tx = reset_tx.clone();
                let packet_tx = packet_tx.clone();
                let media_info = Arc::clone(&media_info);
                move |task_context| {
                    Self::run(
                        path,
                        descriptor,
                        state_tx,
                        reset_tx,
                        packet_tx,
                        media_info,
                        task_context,
                    )
                }
            })
            .await;
        tracing::trace!(name, %path, "started source");

        Ok(Self {
            name: name.to_string(),
            // path,
            descriptor,
            reset_tx,
            packet_tx,
            media_info,
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
            reset_rx: self.reset_tx.subscribe(),
            packet_rx: self.packet_tx.subscribe(),
            media_info: Arc::clone(&self.media_info),
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run(
        path: SourcePath,
        descriptor: MediaDescriptor,
        state_tx: SourceStateTx,
        reset_tx: SourceResetTx,
        packet_tx: SourcePacketTx,
        media_info: Arc<Mutex<Option<MediaInfo>>>,
        mut task_context: TaskContext,
    ) {
        let mut outer_stream_reader = match StreamReader::new(&descriptor).await {
            Ok(stream_reader) => {
                _ = media_info.lock().await.insert(stream_reader.info.clone());
                Some(stream_reader)
            }
            Err(err) => {
                tracing::error!(
                    %err, %descriptor,
                    "failed to start stream",
                );
                None
            }
        };

        'outer: loop {
            let mut stream_reader = match outer_stream_reader {
                Some(stream_reader) => stream_reader,
                None => {
                    'restart: loop {
                        match StreamReader::new(&descriptor).await {
                            Ok(new_stream_reader) => {
                                // Send reset with new media information to listeners so they can
                                // reset their muxers and continue playing.
                                let _ = reset_tx.send(new_stream_reader.info.clone());

                                tracing::info!(%path, "restarted stream");
                                break new_stream_reader;
                            }
                            Err(err) => {
                                tracing::error!(
                                  %err, %descriptor, retry_delay=Self::RETRY_DELAY_SECS,
                                  "failed to restart stream (waiting before retrying)",
                                );
                                // We want to wait some time before retrying. We wrap `wait_for_stop` in
                                // a timeout to achieve this ...
                                match timeout(
                                    time::Duration::from_secs(Self::RETRY_DELAY_SECS),
                                    task_context.wait_for_stop(),
                                )
                                .await
                                {
                                    Ok(()) => {
                                        tracing::trace!(%path, "stopping source (during stream restart)");
                                        // If `wait_for_stop` returns, we break out of the outer loop and stop ...
                                        break 'outer;
                                    }
                                    Err(_) => {
                                        // But if the timeout is reached, we simply restart this loop to try and
                                        // see if we can get the stream reader to work this time.
                                        continue 'restart;
                                    }
                                }
                            }
                        }
                    }
                }
            };
            _ = media_info.lock().await.insert(stream_reader.info.clone());

            'read: loop {
                select! {
                    // CANCEL SAFETY: `StreamReader::read` uses `mpsc::UnboundedReceiver::recv`
                    // internally which is cancel safe.
                    packet = stream_reader.read() => {
                        match packet {
                            Some(Ok(packet)) => {
                                let _ = packet_tx.send(packet.clone());
                            },
                            Some(Err(err)) => {
                                tracing::error!(%path, %err, "failed to read video stream");
                                break 'read;
                            },
                            None => {
                                tracing::error!(%path, "stream reader broken unexpectedly");
                                break 'read;
                            },
                        };
                    },
                    // CANCEL SAFETY: `TaskContext::wait_for_stop` is cancel safe.
                    _ = task_context.wait_for_stop() => {
                        tracing::trace!(%path, "stopping source");
                        stream_reader.stop().await;
                        break 'outer;
                    },
                }
            }

            // Before attempting to restart the stream, instruct the existing (broken)
            // one to stop and wait for it to do so.
            stream_reader.stop().await;

            // Reset the outer stream reader so that it will be reinitialized during
            // the next outer loop cycle.
            outer_stream_reader = None;
        }

        let _ = state_tx.send(SourceState::Stopped(path));
    }
}

pub struct SourceDelegate {
    reset_rx: SourceResetRx,
    packet_rx: SourcePacketRx,
    media_info: Arc<Mutex<Option<MediaInfo>>>,
}

impl SourceDelegate {
    pub async fn media_info(&mut self) -> Option<media::MediaInfo> {
        const MAX_TRIES: usize = 4;
        const TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

        for try_count in 0..MAX_TRIES {
            if try_count > 0 {
                tokio::time::sleep(TIMEOUT).await;
                tracing::trace!(try_count = %try_count + 1, max_tries = MAX_TRIES, "trying again for media info");
            }
            if let Some(media_info) = self.media_info.lock().await.as_ref() {
                return Some(media_info.clone());
            }
        }
        None
    }

    pub fn into_parts(self) -> (SourceResetRx, SourcePacketRx) {
        (self.reset_rx, self.packet_rx)
    }
}

pub type SourcePath = String;
pub type SourcePathRef = str;

pub fn normalize_path(path: SourcePath) -> SourcePath {
    if path.starts_with('/') {
        path
    } else {
        format!("/{}", &path)
    }
}
