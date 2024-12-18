pub mod sdp;
pub mod video;

use video_rs::stream::StreamInfo;

pub use video_rs::Packet;

use std::fmt;
use std::path::PathBuf;

use video_rs::{Error, Location, Reader, Url};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub enum MediaDescriptor {
    Stream(Url),
    File(PathBuf),
}

impl fmt::Display for MediaDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MediaDescriptor::File(path) => write!(f, "file: {}", path.display()),
            MediaDescriptor::Stream(url) => {
                write!(f, "stream: {}", {
                    let mut url_safe = url.clone();
                    let _ = url_safe.set_password(None);
                    url_safe
                })
            }
        }
    }
}

impl fmt::Debug for MediaDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Delegate to default formatting. This also hides the password to
        // prevent passwords from leaking in logs.
        write!(f, "{self}")
    }
}

impl From<MediaDescriptor> for Location {
    fn from(descriptor: MediaDescriptor) -> Self {
        match descriptor {
            MediaDescriptor::File(path) => Location::File(path),
            MediaDescriptor::Stream(url) => Location::Network(url),
        }
    }
}

#[derive(Clone)]
pub struct MediaInfo {
    pub streams: Vec<StreamInfo>,
}

impl MediaInfo {
    pub fn from_reader_best_video_stream(reader: &Reader) -> Result<Self> {
        let best_video_stream_index = reader.best_video_stream_index()?;
        Ok(Self {
            streams: vec![reader.stream_info(best_video_stream_index)?],
        })
    }
}

#[derive(Clone, Default)]
pub struct StreamState {
    pub rtp_seq: u16,
    pub rtp_timestamp: u32,
}
