mod server;

use std::error::Error;

use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  tracing_subscriber::fmt()
    .with_env_filter(tracing_subscriber::EnvFilter::from_env("LOG"))
    .pretty()
    .init();

  let server = Server::new(("localhost", 5554));
  server.run().await
}