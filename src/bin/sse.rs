use std::sync::Arc;

use rust_mcp::mcp::server::Server;


#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let server = Server::new("test", "0.1", 5002);

    let server = Arc::new(server);

    Server::start_sse(server.clone()).await
}
