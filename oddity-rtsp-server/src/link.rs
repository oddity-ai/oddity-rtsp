use tokio::sync::mpsc;

pub struct Sender<I, O> {
  tx: mpsc::UnboundedSender<O>,
  rx: mpsc::UnboundedReceiver<I>,
}

impl<I, O> Sender<I, O> {

  pub async fn command(
    &mut self,
    command: O,
  ) -> Option<I> {
    let _ = self.tx.send(command); // TODO handle err
    self.rx.recv().await
  }

}

pub struct Receiver<I, O> {
  tx: mpsc::UnboundedSender<O>,
  rx: mpsc::UnboundedReceiver<I>,
}

impl<I, O> Receiver<I, O> {

  pub async fn recv(
    &mut self,
  ) -> Option<I> {
    self.rx.recv().await
  }

  pub fn reply(
    &mut self,
    reply: O,
  ) {
    self.tx.send(reply); // TODO handle err
  }

}

pub async fn create<Command, Reply>() -> (
  Sender<Command, Reply>,
  Receiver<Reply, Command>,
) {
  let (tx_cmd, rx_cmd) = mpsc::unbounded_channel();
  let (tx_rpl, rx_rpl) = mpsc::unbounded_channel();
  (
    Sender {
      tx: tx_cmd, 
      rx: rx_rpl,
    },
    Receiver {
      tx: tx_rpl,
      rx: rx_cmd,
    },
  )
}