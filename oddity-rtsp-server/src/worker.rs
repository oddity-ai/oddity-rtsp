use std::thread::{
  spawn,
  JoinHandle,
};

use tokio::sync::oneshot::{
  channel,
  Sender,
  Receiver,
};

/// Represents a worker object. The object manages a detached thread
/// that is dedicated to a single tasks. The main purpose of this
/// object is stop the worker thread when it is dropped.
pub struct Worker {
  handle: Option<JoinHandle<()>>,
  stop_tx: Option<Sender<()>>,
}

impl Worker {

  /// Create a new worker thread and start it.
  /// 
  /// # Arguments
  /// 
  /// * `f` - Closure to run inside thread.
  pub fn new<F>(f: F) -> Self
  where
    F: FnOnce(Stopper) -> (),
    F: Send + 'static,
  {
    let (stop_tx, stop_rx) = channel();
    let handle = spawn(move || {
      f(stop_rx)
    });

    Self {
      handle: Some(handle),
      stop_tx: Some(stop_tx),
    }
  }

  /// Stop the worker manually.
  /// 
  /// Note that the recommended method for stopping the worker is by
  /// dropping the `Worker` object.
  /// 
  /// # Arguments
  /// 
  /// * `wait` - If set to `true`, this function will block until the
  ///   thread stopped itself.
  pub fn stop(mut self, wait: bool) {
    if let Some(handle) = self.handle.take() {
      if let Some(stop_tx) = self.stop_tx.take() {
        if let Ok(()) = stop_tx.send(()) {
          // We take the handle here to make sure the destructor isn't
          // going to be waiting as well. This also allows destructing
          // the worker without waiting at all by calling `stop(false)`
          // before the worker is dropped.
          if wait {
            let _ = handle.join();
          }
        }
      }
    }
  }

}

impl Drop for Worker {

  /// On dropping, send a stop signal to the worker and join.
  fn drop(&mut self) {
    if let Some(handle) = self.handle.take() {
      if let Some(stop_tx) = self.stop_tx.take() {
        if let Ok(()) = stop_tx.send(()) {
          let _ = handle.join();
        }
      }
    }
  }

}

/// Represents the receiver end of the channel that carries the signal
/// to indicate that the worker should stop.
pub type Stopper = Receiver<()>;