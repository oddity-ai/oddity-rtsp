use crossbeam_channel::{
  Receiver,
  RecvError,
  TryRecvError,
};

/// Represents the receiver end of the channel that carries the signal
/// to indicate that the worker should stop.
/// 
/// This wrapper is meant to provide an ergonomic API to the `StopRx`
/// receiver object.
/// 
/// # Example
/// 
/// A `StopRx` object will automatically be passed to a `Service`. This
/// is the simplest possible usage of the `StopRx` object, simply waiting
/// for the parent to signal the service to stop:
/// 
/// ```
/// let service = Service::spawn(move |stop| {
///   stop.wait();
/// });
/// ```
/// 
/// Usually, the caller will do some work either periodically check the
/// `StopRx` or selecting on it:
/// 
/// ```
/// # use std::thread::sleep;
/// # use std::time::Duration;
/// let service = Service::spawn(move |stop| {
///   while !stop.should() {
///     sleep(Duration::from_millis(100));
///   }
/// });
/// ```
pub struct StopRx {
  rx: Receiver<()>,
  flag: bool,
}

impl StopRx {

  /// Check whether or not to stop without blocking. If any previous
  /// call to `should` returned `true`, then any subsequent invocations
  /// will return `true` as well. The same goes for having previously
  /// called `wait`.
  /// 
  /// Note: If the underlying channel fails, this function will signal
  /// the caller to stop.
  pub fn should(&mut self) -> bool {
    if self.flag {
      true
    } else {
      match self.rx.try_recv() {
        Ok(()) => {
          self.flag = true;
          true
        },
        Err(TryRecvError::Disconnected) => {
          tracing::error!("stop channel broke unexpectedly");
          self.flag = true;
          true
        },
        Err(TryRecvError::Empty) => false,
      }
    }
  }

  /// Wait until a stop signal is received. This function is the blocking
  /// equivalent of `should()`.
  /// 
  /// Note: If the underlying channel fails, this function will return.
  pub fn wait(&mut self) {
    match self.rx.recv() {
      Ok(()) => {
        self.flag = true;
      },
      Err(RecvError) => {
        tracing::error!("stop channel broke unexpectedly");
        self.flag = true;
      },
    }
  }

}

impl From<Receiver<()>> for StopRx {

  /// Allows for easy conversion from `Receiver<()>` to `Stop`.
  fn from(receiver: Receiver<()>) -> Self {
    StopRx {
      rx: receiver,
      flag: false,
    }
  }

}