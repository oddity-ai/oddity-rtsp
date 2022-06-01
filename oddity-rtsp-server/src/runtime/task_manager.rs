use std::future::Future;

use tokio::spawn;
use tokio::sync::broadcast;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

pub struct TaskManager {
  hold_tx: Mutex<Option<mpsc::Sender<()>>>,
  hold_rx: Mutex<mpsc::Receiver<()>>,
  stop_tx: broadcast::Sender<()>,
}

impl TaskManager {

  pub fn new() -> Self {
    let (hold_tx, hold_rx) = mpsc::channel(1);
    let (stop_tx, _) = broadcast::channel(1);
    Self {
      // Must protect by mutex since another task might
      // invalidate the `hold_tx` once shutdown begins.
      hold_tx: Mutex::new(Some(hold_tx)),
      // Must protect `hold_rx` by mutex to allow for
      // internal mutability.
      hold_rx: Mutex::new(hold_rx),
      stop_tx,
    }
  }

  pub async fn spawn<F, T>(
    &self,
    f: F,
  )
  where
    F: FnOnce(TaskContext) -> T + Send + 'static,
    T: Future + Send + 'static,
    T::Output: Send + 'static,
  {
    // Unlock the `hold_tx` mutex and grab a copy to pass
    // on to the task context. `hold_tx` could be empty if
    // another task already asked the manager to stop, in
    // which case we ignore the request and don't start a
    // task at all.
    if let Some(hold_tx) = self
        .hold_tx
        .lock()
        .await
        .as_ref()
        .map(|hold_tx| hold_tx.clone()) {
      let stop_rx = self.stop_tx.subscribe();
      let _ = spawn(
        async move {
          // Instantiate task context here. After the fut-
          // ure genrated by `f` has finished, it will be
          // dropped automatically, which will cause the
          // hold to be released as well.
          let task_context = TaskContext {
            _token: hold_tx,
            stop: stop_rx,
          };

          f(task_context).await;
        }
      );
    }
  }

  pub async fn stop(&self) {
    // If we don't drop the apex hold_tx here then the call
    // to recv() below will block forever since there would
    // be one remaing hold.
    drop(self.hold_tx.lock().await.take());

    // Send stop signal to all tasks using the stop signal
    // broadcast channel; tasks must respond! Note: It is
    // very important that this happens after the remaining
    // `hold_tx` is dropped, since dropping `hold_tx` also
    // causes futher invocations of `spawn` to be ignored.
    // If they were not ignored, then a new task could pot-
    // entially be spawned that never received the stop re-
    // quest which would cause a deadlock.
    let _ = self.stop_tx.send(());

    // Wait for the channel to break after all `hold_tx` are
    // dropped, which means all tasks have finished.
    let _ = self.hold_rx.lock().await.recv().await;
  }

}

pub struct TaskContext {
  stop: broadcast::Receiver<()>,
  _token: mpsc::Sender<()>,
}

impl TaskContext {

  pub async fn wait_for_stop(&mut self) {
    let _ = self.stop.recv().await;
  }

}