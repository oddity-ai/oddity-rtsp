//! Async wrapper functions for [`oddity_video::Reader`].

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