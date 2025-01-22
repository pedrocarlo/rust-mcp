use crate::mcp::schema::{self, JSONRPCMessage};

use super::error::{ApiError, Result};
use super::utils::create_error_response;
use super::InitializeStatus;
use super::{Server, SessionId};

pub fn handle_request(
    server: &Server,
    request: &schema::JSONRPCRequest,
    session_id: &SessionId,
) -> Result<JSONRPCMessage> {
    {
        let map = server
            .clients
            .write()
            .or_else(|_| Err(ApiError::PoisonedLock))?;

        let mut client_conn = map
            .get(session_id)
            .ok_or(ApiError::MissingClient)?
            .lock()
            .or_else(|_| Err(ApiError::PoisonedLock))?;

        if let schema::RequestParams::Initialize(ref init) = request.params {
            match client_conn.initialize_status {
                InitializeStatus::NotInitialized => {
                    client_conn.initialize_status = InitializeStatus::Initializing;
                    client_conn.capabilities = init.capabilities.clone();
                }
                InitializeStatus::Initializing => {
                    return Ok(create_error_response(
                        &request.id,
                        schema::INVALID_REQUEST,
                        "Connection already initializing",
                    ))
                }
                InitializeStatus::Initialized => {
                    return Ok(create_error_response(
                        &request.id,
                        schema::INVALID_REQUEST,
                        "Connection already initialized",
                    ))
                }
            };
        } else {
            match client_conn.initialize_status {
                InitializeStatus::NotInitialized => {
                    return Ok(create_error_response(
                        &request.id,
                        schema::INVALID_REQUEST,
                        "Connection not initialized",
                    ))
                }
                _ => (),
            };
        }
    }
    match &request.params {
        schema::RequestParams::Initialize(init) => {
            let response = handle_initialize(server, init, session_id, &request.id);

            Ok(response)
        }
        _ => unimplemented!(),
    }
}

pub fn handle_initialize(
    server: &Server,
    _request: &schema::InitializeRequestParams,
    _session_id: &SessionId,
    id: &schema::RequestId,
) -> JSONRPCMessage {
    let initialize_result = schema::JSONRPCResult {
        id: id.to_owned(),
        json_rpc: schema::JSONRPC_VERSION.into(),
        result: schema::Result {
            base: schema::ResultBase::default(),
            defined_fields: schema::ResultEnum::Initialize(schema::InitializeResult {
                protocol_version: schema::LATEST_PROTOCOL_VERSION.into(),
                capabilities: server.capabilities.clone(),
                server_info: schema::Implementation {
                    name: server.name.to_owned(),
                    version: server.name.to_owned(),
                },
                instructions: None,
            }),
        },
    };

    JSONRPCMessage::Response(schema::JSONRPCResponse::Result(initialize_result))
}
