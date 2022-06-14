//! Async wrapper functions for [`oddity_video::Reader`].

use futures::Stream;
use futures::stream;

use tokio::task;

use oddity_video::{self as video, Reader};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_reader_with_sane_settings(
  locator: video::Locator,
) -> Result<Reader> {
  task::spawn_blocking(move || {
      let options = match locator {
        video::Locator::Path(_) => {
          Default::default()
        },
        video::Locator::Url(_) => {
          // For streaming sources (live sources), we want to use TCP transport
          // over UDP and have sane timeouts.
          video::Options::new_with_rtsp_transport_tcp_and_sane_timeouts()
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

pub fn into_stream(
  reader: Reader,
  stream_index: usize,
) -> impl Stream<Item=Result<video::Packet>> {
  stream::unfold(reader, move |mut local_reader| async move {
    let (packet, reader) = task::spawn_blocking(move || {
        let packet = local_reader.read(stream_index);
        (packet, local_reader)
      })
      .await
      .unwrap();

    match packet {
      Err(video::Error::ReadExhausted) => {
        // If we reached EOF, map to `None` value
        None
      },
      _ => {
        Some((packet, reader))
      }
    }
  })
}