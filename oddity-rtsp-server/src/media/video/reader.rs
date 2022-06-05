//! Async wrapper functions for [`oddity_video::Reader`].

use futures::Stream;
use futures::stream;

use tokio::task;

use oddity_video::{self as video, Reader};

type Result<T> = std::result::Result<T, video::Error>;

pub async fn make_reader(
  locator: video::Locator,
) -> Result<Reader> {
  task::spawn_blocking(move || {
      Reader::new(&locator)
    })
    .await
    .unwrap()
}

pub fn into_stream(
  reader: Reader,
  stream_index: usize,
) -> impl Stream<Item=Result<video::Packet>> {
  stream::unfold(reader, move |mut local_reader| async move {
    // TODO maybe map end-of-stream to `None` here and handle
    // appropriately
    let (packet, reader) = task::spawn_blocking(move || {
        let packet = local_reader.read(stream_index);
        (packet, local_reader)
      })
      .await
      .unwrap();
    Some((packet, reader))
  })
}