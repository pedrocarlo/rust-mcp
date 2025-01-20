mod sse;
mod stdio;

use serde_json::ser;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::mcp::schema;

pub struct Message {
    pub session_id: SessionId,
    pub sse_message: schema::JSONRPCMessage,
}

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
    clients: Arc<RwLock<HashMap<SessionId, Sender<Message>>>>,
    send_close_client: Sender<SessionId>,
    name: String,
    version: String,
    capabilities: schema::ServerCapabilities,
}

impl Server {
    // TODO maybe faster and more memory efficient to just clone th
    fn new(name: &str, version: &str, port: usize, send: Sender<SessionId>) -> Self {
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
            send_close_client: send,
        }
    }

    fn new_connection(&self, session_id: &str) -> Client {
        let (send, recv): (Sender<Message>, Receiver<Message>) = mpsc::channel(32);

        self.clients
            .write()
            .unwrap()
            .insert(session_id.to_string(), send);

        Client::new(session_id, recv)
    }

    fn close_connection(&self, session_id: SessionId) {
        // TODO later handler error where you cannot write to map
        self.clients.write().unwrap().remove(&session_id);
    }

    async fn listen(
        clients: Arc<RwLock<HashMap<SessionId, Sender<Message>>>>,
        recv_close_client: Receiver<String>,
    ) {
        let mut rx = recv_close_client;
        loop {
            tokio::select! {
                Some(session_id) = rx.recv() => {
                    // TODO lock can be poisoned here
                    if let Some(mut map) = clients.write().ok() {
                        map.remove(&session_id);
                    }
                },
            };
        }
    }

    /// Starts an SSE Server. Moves ownership to function and blocks
    pub async fn serve_sse(
        name: &str,
        version: &str,
        port: usize,
        endpoint: &str,
    ) -> Result<(), std::io::Error> {
        let (send, recv) = mpsc::channel(32);

        let server = Server {
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
            send_close_client: send,
        };

        let clients = server.clients.clone();
        tokio::spawn(async move { Server::listen(clients, recv) });

        sse::serve(server, endpoint).await
        // SseServer::start(server, endpoint).await
    }
}

#[derive(Debug)]
struct Client {
    recv: Receiver<Message>,
    session_id: SessionId,
    capabilities: Option<schema::ClientCapabilities>,
    initialize_status: InitializeStatus,
}

impl Client {
    fn new(session_id: &str, recv: Receiver<Message>) -> Self {
        Self {
            session_id: String::from(session_id),
            recv,
            initialize_status: InitializeStatus::default(),
            capabilities: None,
        }
    }
}
