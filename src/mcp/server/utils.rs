use crate::mcp::schema::{self, JSONRPCError, JSONRPCMessage};
use axum::http::StatusCode;
use axum_derive_error::ErrorResponse;
use thiserror::Error;

pub fn create_error_response(id: &schema::RequestId, code: i64, message: &str) -> JSONRPCMessage {
    let err = JSONRPCError {
        json_rpc: schema::JSONRPC_VERSION.into(),
        id: id.to_owned(),
        error: schema::ErrorParams {
            code,
            message: message.into(),
            data: None,
        },
    };

    JSONRPCMessage::Response(schema::JSONRPCResponse::Error(err))
}

#[derive(Error, ErrorResponse)]
pub enum CustomErrors {
    #[error("Poisoned Lock")]
    PoisonedLock,
}
