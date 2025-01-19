mod sse;
mod stdio;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::mcp::schema;

use sse::SseServer;

#[derive(Debug, Default)]
enum InitializeStatus {
    #[default]
    NotInitialized,
    Initializing,
    Initialized,
}

type SessionId = String;

#[derive(Debug)]
pub struct Server {
    port: usize,
    clients: Arc<RwLock<HashMap<SessionId, Sender<String>>>>,
    name: String,
    version: String,
    capabilities: schema::ServerCapabilities,
}

impl Server {
    // TODO maybe faster and more memory efficient to just clone th
    pub fn new(name: &str, version: &str, port: usize) -> Self {
        Self {
            name: String::from(name),
            version: String::from(version),
            port,
            capabilities: schema::ServerCapabilities {
                experimental: None,
                logging: None,
                prompts: None,
                resources: None,
                tools: None,
            },
            clients: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn new_connection(&self, session_id: &str) -> Client {
        let (send, recv): (Sender<String>, Receiver<String>) = mpsc::channel(32);

        self.clients
            .write()
            .unwrap()
            .insert(session_id.to_string(), send);

        Client::new(session_id, recv)
    }

    async fn listen(&self) {
        loop {
            // tokio::select! {
            //     msg = &mut recv => {
            //         println!("Got message: {}", msg.unwrap());
            //         break;
            //     }
            // }
        }
    }

    /// Starts an SSE Server. Moves ownership to function and blocks
    pub async fn start_sse(server: Arc<Server>) -> Result<(), std::io::Error> {
        let server_clone = server.clone();
        tokio::spawn(async move {
            server_clone.listen().await;
        });

        let sse_server = SseServer::new("sse", server.clone());
        sse_server.start().await
    }
}

#[derive(Debug)]
struct Client {
    recv: Receiver<String>,
    session_id: SessionId,
    capabilities: Option<schema::ClientCapabilities>,
    initialize_status: InitializeStatus,
}

impl Client {
    fn new(session_id: &str, recv: Receiver<String>) -> Self {
        Self {
            session_id: String::from(session_id),
            recv,
            initialize_status: InitializeStatus::default(),
            capabilities: None,
        }
    }
}
