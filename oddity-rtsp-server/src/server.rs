use futures::StreamExt;

use tokio::net::TcpListener;
use tokio_util::codec::Decoder;

use oddity_rtsp_protocol::{
  Codec,
  AsServer,
};

pub struct Server {

}

impl Server {

  // TODO fix error type
  pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:554").await?;

    loop {
      let (socket, addr) = listener.accept().await?;

      tokio::spawn(async move {
        let mut framed = Codec::<AsServer>::new().framed(socket);
        while let Some(Ok(request)) = framed.next().await {
          println!("{:?}", request); //TODO
        }
      });
    }
  }

}