pub use crossbeam_channel::*;

pub fn default<T>()
  -> (Sender<T>, Receiver<T>)
{
  bounded(1024)
}