mod routes;

use async_stream::try_stream;
use axum::{
    response::sse::{Event, Sse},
    routing::get,
    Router,
};
use futures::stream::Stream;
use std::{convert::Infallible, sync::Arc, time::Duration};
use uuid::Uuid;

use super::{Server, SessionId};

// Sse Server should live as long as mcp_server
// But mcp_server can live longer

pub struct SseServer {
    endpoint: String,
    mcp_server: Arc<Server>,
}

impl SseServer {
    pub fn new(endpoint: &str, mcp_server: Arc<Server>) -> Self {
        Self {
            endpoint: String::from(endpoint),
            mcp_server,
        }
    }

    fn routes(&self) -> Router {
        let mcp_server = self.mcp_server.clone();
        let endpoint = self.endpoint.to_string();
        let app = Router::new().route("/", get(move || sse_handler(mcp_server, endpoint)));
        app
    }

    pub async fn start(&self) -> Result<(), std::io::Error> {
        let port = &self.mcp_server.port;
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
            .await
            .unwrap();
        axum::serve(listener, self.routes()).await
    }
}

async fn sse_handler(
    mcp_server: Arc<Server>,
    endpoint: String,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    use querystring::stringify;

    let session_id: SessionId = Uuid::new_v4().to_string();

    let mut client = {
        // Using block here so that lock can be dropped
        mcp_server.new_connection(&session_id)
    };

    let session_uri = format!(
        "{}?{}",
        endpoint,
        stringify(vec![("sessionId", &session_id)])
    );

    let mut endpoint_sent = false;

    let stream = try_stream! {
        loop {
            if !endpoint_sent {
                endpoint_sent = true;
                yield Event::default().event("endpoint").data(session_uri.clone())
            } else {
                match client.recv.recv().await {
                    Some(v) => yield Event::default().event("message").data(v),
                   None => (),
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
