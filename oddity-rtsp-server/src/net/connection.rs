use tokio::select;
use tokio::sync::oneshot;
use tokio::net;
use tokio_stream::StreamExt;
use tokio_util::codec;

use oddity_rtsp_protocol::{Codec, AsServer};

use crate::runtime::Runtime;
use crate::runtime::task_manager::TaskContext;

type DisconnectTx = oneshot::Sender<()>;
type DisconnectRx = oneshot::Receiver<()>;

pub struct Connection {
  // TODO disconnect handling
}

impl Connection {

  pub async fn start(
    inner: net::TcpStream,
    runtime: &Runtime,
  ) -> Self {
    runtime
      .task()
      .spawn(move |task_context| Self::run(inner, task_context));

    Connection {
    }
  }

  async fn run(
    inner: net::TcpStream,
    mut task_context: TaskContext,
  ) {
    let (read, write) = inner.into_split();
    let mut inbound = codec::FramedRead::new(read, Codec::<AsServer>::new());
    let mut outbound = codec::FramedWrite::new(write, Codec::<AsServer>::new());

    loop {
      select! {
        packet = inbound.next() => {

        },
        _ = task_context.wait_for_stop() => {
          break;
        },
      };
    }
  }

  // TODO writer task!

}