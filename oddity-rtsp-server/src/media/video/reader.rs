use std::thread;

use tokio::sync::mpsc;
use tokio::task;

use video_rs as video;

use crate::media::{MediaDescriptor, MediaInfo};

type Result<T> = std::result::Result<T, video::Error>;

pub struct StreamReader {
    pub info: MediaInfo,
    handle: Option<thread::JoinHandle<()>>,
    packet_rx: mpsc::UnboundedReceiver<Result<video::Packet>>,
    stop_tx: mpsc::UnboundedSender<()>,
}

impl StreamReader {
    pub async fn new(descriptor: &MediaDescriptor) -> Result<Self> {
        let is_file = matches!(descriptor, MediaDescriptor::File(_));

        tracing::trace!(%descriptor, "initializing reader");
        let inner = backend::make_reader_with_sane_settings(descriptor.clone().into()).await?;
        tracing::trace!(%descriptor, "initialized reader");

        let info = MediaInfo::from_reader_best_video_stream(&inner)?;
        let stream_index = info.streams[0].index;
        tracing::trace!(%descriptor, stream_index=stream_index, "selected video stream");

        let (packet_tx, packet_rx) = mpsc::unbounded_channel();
        let (stop_tx, stop_rx) = mpsc::unbounded_channel();

        tracing::trace!(%descriptor, "starting stream reader");
        let handle =
            thread::spawn(move || Self::run(inner, stream_index, packet_tx, stop_rx, is_file));
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
        let mut times = Times::new();

        loop {
            match stop_rx.try_recv() {
                Ok(()) | Err(mpsc::error::TryRecvError::Disconnected) => {
                    tracing::trace!("stopping stream reader");
                    break;
                }
                Err(mpsc::error::TryRecvError::Empty) => {}
            };

            let read = reader.read(stream_index);

            if is_file {
                // To pretend the file is a live stream, we need to wait a bit after
                // each packet or we'll overload the consumer.
                if let Ok(packet) = read.as_ref() {
                    thread::sleep(packet.duration().into());
                }
            }

            let packet = match read {
                // Forward OK packets.
                Ok(mut packet) => {
                    // Manually keep time for file-based streams. This way we can seek
                    // in the file and pretend that time is still running linearly.
                    if is_file {
                        times.update(&mut packet);
                    }

                    Some(Ok(packet))
                }
                // If the error was caused by an exhausted stream, try and see if we
                // can seek to the beginning of the file and then just keep reading:
                // we don't send a packet and just continue the loop in that case. If
                // seeking fails, forward the error.
                Err(video::Error::ReadExhausted) => {
                    tracing::trace!("seeking to beginning of file after stream exhausted");
                    match reader.seek_to_start() {
                        Ok(()) => None,
                        Err(err) => {
                            tracing::error!(%err, "failed to seek to beginning of file");
                            Some(Err(err))
                        }
                    }
                }
                // Forward any errors.
                Err(err) => Some(Err(err)),
            };

            if let Some(packet) = packet {
                if packet_tx.send(packet).is_err() {
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

struct Times {
    next_dts: video::Time,
    next_pts: video::Time,
}

impl Times {
    pub fn new() -> Self {
        Times {
            next_dts: video::Time::zero(),
            next_pts: video::Time::zero(),
        }
    }

    pub fn update(&mut self, packet: &mut video::Packet) {
        if packet.duration().has_value() {
            packet.set_dts(&self.next_dts);
            packet.set_pts(&self.next_pts);
            self.next_dts = self.next_dts.aligned_with(&packet.duration()).add();
            self.next_pts = self.next_pts.aligned_with(&packet.duration()).add();
        }
    }
}

// Holds functions that deal with the video backend stuff in `video_rs`.
pub mod backend {

    use tokio::task;

    use video_rs::{Error, Location, Options, Reader, ReaderBuilder};

    pub async fn make_reader_with_sane_settings(location: Location) -> Result<Reader, Error> {
        task::spawn_blocking(move || {
            let options = match location {
                Location::File(_) => Default::default(),
                Location::Network(_) => {
                    // For streaming sources (live sources), we want to use TCP transport
                    // over UDP and have sane timeouts.
                    Options::preset_rtsp_transport_tcp_and_sane_timeouts()
                }
            };

            ReaderBuilder::new(location).with_options(&options).build()
        })
        .await
        .unwrap()
    }
}
