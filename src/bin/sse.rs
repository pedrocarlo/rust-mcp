use rust_mcp::mcp::server::Server;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    Server::serve_sse("test", "0.1", 3001, "messages").await
}
