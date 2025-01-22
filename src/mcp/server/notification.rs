use crate::mcp::schema;
use crate::mcp::server::error::ApiError;

use super::error::Result;
use super::InitializeStatus;
use super::{Server, SessionId};

pub fn handle_notification(
    server: &Server,
    request: &schema::JSONRPCNotification,
    session_id: &SessionId,
) -> Result<()> {
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

        match request.params {
            schema::NotificationParams::Initialized(_) => {
                client_conn.initialize_status = InitializeStatus::Initialized;
            }
            _ => todo!(),
        }
    }
    todo!()
}
