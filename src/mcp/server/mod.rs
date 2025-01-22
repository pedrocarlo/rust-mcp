mod error;
mod notification;
mod request;
mod sse;
mod stdio;
mod utils;

use error::{ApiError, Result};
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
    clients: Arc<RwLock<HashMap<SessionId, Arc<Mutex<ClientConn>>>>>,
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

    fn new_connection(&self, session_id: &str) -> Result<Client> {
        let (send, recv): (Sender<Message>, Receiver<Message>) = mpsc::channel(32);

        {
            // Drop lock faster
            self.clients
                .write()
                .or_else(|_| Err(ApiError::PoisonedLock))?
                .insert(
                    session_id.to_string(),
                    Arc::new(Mutex::new(ClientConn::new(session_id, send, None))),
                );
        }

        Ok(Client::new(session_id, recv))
    }

    fn close_connection(&self, session_id: SessionId) -> Result<()> {
        tracing::debug!("close client connection");

        // TODO later handler error where you cannot write to map
        self.clients
            .write()
            .or_else(|_| Err(ApiError::PoisonedLock))?
            .remove(&session_id);

        {
            let len = self
                .clients
                .read()
                .or_else(|_| Err(ApiError::PoisonedLock))?
                .len();
            tracing::debug!("client_map_size" = len);
        }

        Ok(())
    }

    async fn listen(
        clients: Arc<RwLock<HashMap<SessionId, Arc<Mutex<ClientConn>>>>>,
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
    pub async fn serve_sse(name: &str, version: &str, port: usize, endpoint: &str) -> Result<()> {
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
    }
}

#[derive(Debug)]
struct Client {
    recv: Receiver<Message>,
    session_id: SessionId,
}

impl Client {
    fn new(session_id: &str, recv: Receiver<Message>) -> Self {
        Self {
            session_id: String::from(session_id),
            recv,
        }
    }
}



#[derive(Debug)]
struct ClientConn {
    session_id: SessionId,
    initialize_status: InitializeStatus,
    send: Sender<Message>,
    capabilities: schema::ClientCapabilities,
    protocol_version: schema::ProtocolVersion,
}

impl ClientConn {
    fn new(
        session_id: &str,
        send: Sender<Message>,
        capabilities: Option<schema::ClientCapabilities>,
    ) -> Self {
        Self {
            session_id: session_id.to_string(),
            initialize_status: InitializeStatus::default(),
            send,
            capabilities: capabilities.unwrap_or_default(),
            protocol_version: schema::ProtocolVersion::default(),
        }
    }
}
