use tokio::sync::mpsc;

// TODO Tx rename
// TODO Rx rename
pub struct Sender<Tx, Rx> {
  tx: mpsc::Sender<Tx>,
  rx: mpsc::Receiver<Rx>,
}

impl<Tx, Rx> Sender<Tx, Rx> {

  pub async fn command(
    &mut self,
    command: Tx,
  ) -> Option<Rx> {
    let _ = self.tx.send(command).await; // TODO
    self.rx.recv().await
  }

}

pub struct Receiver<Tx, Rx> {
  tx: mpsc::Sender<Tx>,
  rx: mpsc::Receiver<Rx>,
}

impl<Tx, Rx> Receiver<Tx, Rx> {

  pub async fn recv(
    &mut self,
  ) -> Option<Rx> {
    self.rx.recv().await
  }

  pub async fn reply(
    &mut self,
    reply: Tx,
  ) {
    self.tx.send(reply).await; // TODO handling
  }

}

pub async fn create<Command, Reply>() -> (Sender<Command, Reply>, Receiver<Reply, Command>) {

}