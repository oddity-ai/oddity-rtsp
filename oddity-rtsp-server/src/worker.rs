use std::thread::{
  spawn,
  JoinHandle,
};

use tokio::sync::oneshot::{
  channel,
  Sender,
  Receiver,
};

// TODO refactor into crate

pub struct Worker {
  handle: Option<JoinHandle<()>>,
  stop_tx: Option<Sender<()>>,
}

impl Worker {

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

pub type Stopper = Receiver<()>;