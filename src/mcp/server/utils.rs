use crate::mcp::schema::{self, JSONRPCError, JSONRPCMessage};

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
