use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::mcp::schema::*;

#[derive(Serialize, Deserialize)]
struct TestProgress {
    #[serde(rename = "progressToken")]
    progress_token: ProgressToken,
}

#[test]
fn progress_token_deserialize() {
    let data = r#"
        {
            "progressToken": "hi"
        }"#;

    let test_progress: TestProgress = serde_json::from_str(&data).unwrap();

    match test_progress.progress_token {
        ProgressToken::Number(num) => panic!("Progress Token should be a string but got {num}"),
        ProgressToken::String(val) => assert_eq!(val, "hi"),
    }
}

#[test]
fn initialize_message_deserialize() {
    let correct_msg = JSONRPCRequest {
        id: RequestId::Number(0),
        json_rpc: "2.0".to_string(),
        params: RequestParams::Initialize(InitializeRequestParams {
            protocol_version: ProtocolVersion::Mcp2024_11_05,
            capabilities: ClientCapabilities {
                experimental: None,
                roots: Some(RootCapabilities { list_changed: None }),
                sampling: Some(HashMap::new()),
            },
            client_info: Implementation {
                version: "0.0.1".to_string(),
                name: "mcp-inspector".to_string(),
            },
        }),
    };

    // println!("{}", serde_json::to_string_pretty(&correct_msg).unwrap());

    let data = json!({
            "jsonrpc": "2.0",
            "id": 0,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "sampling": {}, "roots": {} },
                "clientInfo": { "name": "mcp-inspector", "version": "0.0.1" }
            }
        }
    )
    .to_string();

    let message: JSONRPCRequest = serde_json::from_str(&data).unwrap();

    assert_eq!(message, correct_msg);
}

#[test]
fn initialized_notification_deserialize() {
    let correct_msg = JSONRPCNotification {
        json_rpc: "2.0".to_string(),
        params: NotificationParams::Initialized(InitializedNotificationParams {
            notification_base: NotificationBaseParams {
                meta: None,
                extra: HashMap::new(),
            },
        }),
    };

    // println!("{}", serde_json::to_string_pretty(&correct_msg).unwrap());

    let data = json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
        }
    )
    .to_string();

    let message: JSONRPCNotification = serde_json::from_str(&data).unwrap();

    assert_eq!(message, correct_msg);
}
