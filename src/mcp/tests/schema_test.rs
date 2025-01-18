use serde::{Deserialize, Serialize};

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
