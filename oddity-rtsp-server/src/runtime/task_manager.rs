use std::future::Future;

use tokio::spawn;
use tokio::sync::broadcast;
use tokio::sync::mpsc;

pub struct TaskManager {
  stop_tx: broadcast::Sender<()>,
  hold_tx: mpsc::Sender<()>,
  hold_rx: mpsc::Receiver<()>,
}

impl TaskManager {

  pub fn new() -> Self {
    let (stop_tx, _) = broadcast::channel(1);
    let (hold_tx, hold_rx) = mpsc::channel(1);
    Self {
      stop_tx,
      hold_tx,
      hold_rx,
    }
  }

  pub fn spawn<F, T>(
    &self,
    f: F,
  )
  where
    F: FnOnce(TaskContext) -> T + Send + 'static,
    T: Future + Send + 'static,
    T::Output: Send + 'static,
  {
    let stop_rx = self.stop_tx.subscribe();
    let hold_tx = self.hold_tx.clone();

    let _ = spawn(
      async move {
        let task_context = TaskContext {
          stop: stop_rx,
          _token: hold_tx,
        };

        f(task_context).await;
      }
    );
  }

  pub async fn stop(mut self) {
    // send stop signal to all tasks using the stop signal
    // broadcast channel; tasks must respond!
    let _ = self.stop_tx.send(());
    // if we don't drop the apex hold_tx here then the call
    // to recv() below will block forever
    drop(self.hold_tx);
    // wait for the channel to break
    let _ = self.hold_rx.recv().await;
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