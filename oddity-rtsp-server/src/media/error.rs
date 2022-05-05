use oddity_video::Error as MediaError;

pub enum Error {
  CodecNotSupported,
  Media(MediaError),
}