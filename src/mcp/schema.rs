use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    fmt::{self, Display},
};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum JSONRPCMessage {
    Request(JSONRPCRequest),
    Notification(JSONRPCNotification),
    Response(JSONRPCResponse),
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolVersion {
    #[serde(rename = "2024-11-05")]
    #[default]
    Mcp2024_11_05,
}

// impl Display for ProtocolVersion {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(
//             f,
//             "{}",
//             match self {
//                 Self::Mcp2024_11_05 => "2024-11-05".to_string(),
//             }
//         )
//     }
// }

pub const LATEST_PROTOCOL_VERSION: &ProtocolVersion = &ProtocolVersion::Mcp2024_11_05;

pub const JSONRPC_VERSION: &str = "2.0";

// TODO see where to implement _meta for request and result types

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum ProgressToken {
    String(String),
    Number(i64),
}

pub type Cursor = String;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequestBaseMeta {
    #[serde(skip_serializing_if = "Option::is_none")]
    progress_token: Option<ProgressToken>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RequestBaseParams {
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<RequestBaseMeta>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationBaseParams {
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResultBase {
    #[serde(rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<HashMap<String, Value>>,

    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Result {
    #[serde(flatten)]
    pub base: ResultBase,

    #[serde(flatten)]
    pub defined_fields: ResultEnum,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JSONRPCRequest {
    #[serde(flatten)]
    pub params: RequestParams,
    #[serde(rename = "jsonrpc")]
    pub json_rpc: String,
    pub id: RequestId,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JSONRPCNotification {
    #[serde(flatten)]
    pub params: NotificationParams,
    #[serde(rename = "jsonrpc")]
    pub json_rpc: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JSONRPCResult {
    #[serde(rename = "jsonrpc")]
    pub json_rpc: String,
    pub id: RequestId,
    pub result: Result,
}

pub const PARSE_ERROR: i64 = -32700;
pub const INVALID_REQUEST: i64 = -32600;
pub const METHOD_NOT_FOUND: i64 = -32601;
pub const INVALID_PARAMS: i64 = -32602;
pub const INTERNAL_ERROR: i64 = -32603;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JSONRPCError {
    #[serde(rename = "jsonrpc")]
    pub json_rpc: String,
    pub id: RequestId,
    pub error: ErrorParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorParams {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Default for ErrorParams {
    fn default() -> Self {
        Self {
            code: INTERNAL_ERROR,
            message: "Unknown error ocurred".to_string(),
            data: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum JSONRPCResponse {
    Result(JSONRPCResult),
    Error(JSONRPCError),
}

pub type EmptyResult = ResultBase;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CancelledNotificationParams {
    pub request_id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestParams {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ClientCapabilities,
    pub client_info: Implementation,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResult {
    pub protocol_version: ProtocolVersion,
    pub capabilities: ServerCapabilities,
    pub server_info: Implementation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InitializedNotificationParams {
    #[serde(flatten)]
    pub notification_base: NotificationBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roots: Option<RootCapabilities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RootCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experimental: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<HashMap<String, Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribe: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Implementation {
    pub name: String,
    pub version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PingRequestParams {
    #[serde(flatten)]
    pub request_base: RequestBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProgressNotificationParams {
    pub progress_token: ProgressToken,
    pub progress: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedRequestParams {
    #[serde(flatten)]
    pub request_base: RequestBaseParams,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_cursor: Option<Cursor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesRequestParams {
    #[serde(flatten)]
    pub paginated_base: PaginatedRequestParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesResult {
    #[serde(flatten)]
    // Composition with flattening to emulate schema inheritance
    pub paginated_base: PaginatedResult,
    pub resources: Vec<Resource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListResourceTemplatesRequestParams {
    #[serde(flatten)]
    pub paginated_base: PaginatedRequestParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListResourcesTemplateResult {
    #[serde(flatten)]
    // Composition with flattening to emulate schema inheritance
    pub paginated_base: PaginatedResult,
    pub resources_templates: Vec<ResourceTemplate>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceRequestParams {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ReadResourceResult {
    pub contents: Vec<ContentsResource>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ContentsResource {
    Text(TextResourceContents),
    Blob(BlobResourceContents),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceListChangedNotificationParams {
    #[serde(flatten)]
    pub notification_base: NotificationBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SubscribeRequestParams {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UnsubscribeRequestParams {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceUpdatedNotificationParams {
    pub uri: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(flatten)]
    pub annotated_base: AnnotatedBase,

    pub uri: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemplate {
    #[serde(flatten)]
    annotated_base: AnnotatedBase,

    uri_template: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    mime_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextResourceContents {
    #[serde(flatten)]
    resource_contents_base: ResourceContents,

    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BlobResourceContents {
    #[serde(flatten)]
    resource_contents_base: ResourceContents,

    blob: String,
}

// Prompts

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsRequestParams {
    #[serde(flatten)]
    paginated_base: PaginatedRequestParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListPromptsResult {
    #[serde(flatten)]
    paginated_base: PaginatedResult,

    prompts: Vec<Prompt>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetPromptRequestParams {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<HashMap<String, String>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GetPromptResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    messages: Vec<PromptMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<Vec<PromptArgument>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptArgument {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    User,
    Assistant,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptMessage {
    role: Role,
    content: PromptMessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PromptMessageContent {
    Text(TextContent),
    Image(ImageContent),
    #[serde(rename = "resource")]
    Embedded(EmbeddedResource),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddedResource {
    #[serde(flatten)]
    annotated_base: AnnotatedBase,

    resource: EmbeddedResourceEnum,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum EmbeddedResourceEnum {
    Text(TextResourceContents),
    Blob(BlobResourceContents),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PromptListChangedNotificationParams {
    #[serde(flatten)]
    notification_base: NotificationBaseParams,
}

// Tools

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsRequestParams {
    #[serde(flatten)]
    paginated_base: PaginatedRequestParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListToolsResult {
    #[serde(flatten)]
    paginated_base: PaginatedResult,
    tools: Vec<Tool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CallToolResult {
    content: Vec<CallToolContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_error: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CallToolContent {
    Text(TextContent),
    Image(ImageContent),
    #[serde(rename = "resource")]
    Embedded(EmbeddedResource),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CallToolRequestParams {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    arguments: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolListChangedNotificationParams {
    #[serde(flatten)]
    notifications_base: NotificationBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    input_schema: ToolInputSchemaType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    properties: Option<HashMap<String, Value>>,
    required: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ToolInputSchemaType {
    Object(ToolInputSchema),
}

// Logging

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SetLevelRequestParams {
    level: LoggingLevel,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LoggingMessageNotificationParams {
    level: LoggingLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    logger: Option<String>,
    data: Value, // TODO maybe Option<Value>
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum LoggingLevel {
    Debug,
    Info,
    Notice,
    Warning,
    Error,
    Critical,
    Alert,
    Emergency,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageRequestParams {
    messages: Vec<SamplingMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model_preferences: Option<ModelPreferences>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<OrderedFloat<f32>>, // TODO maybe validate between 0 and 1
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Value>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateMessageResult {
    #[serde(flatten)]
    sampling_message: SamplingMessage,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_reason: Option<StopReason>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum StopReason {
    EndTurn,
    StopSequence,
    MaxTokens,
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SamplingMessage {
    role: Role,
    content: SamplingMessageContent,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum SamplingMessageContent {
    Text(TextContent),
    Image(ImageContent),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AnnotatedBase {
    #[serde(skip_serializing_if = "Option::is_none")]
    annotations: Option<Annotations>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Annotations {
    #[serde(skip_serializing_if = "Option::is_none")]
    audience: Option<Vec<Role>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TextContent {
    #[serde(flatten)]
    annotated_base: AnnotatedBase,
    text: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImageContent {
    #[serde(flatten)]
    annotated_base: AnnotatedBase,
    data: String,
    mime_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelPreferences {
    #[serde(skip_serializing_if = "Option::is_none")]
    hints: Option<Vec<ModelHint>>,
    cost_priority: OrderedFloat<f32>, // TODO validation here min val = 0, max val = 1
    speed_priority: OrderedFloat<f32>, // TODO validation here min val = 0, max val = 1
    intelligence_priority: OrderedFloat<f32>, // TODO validation here min val = 0, max val = 1
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelHint {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

// Autocomplete

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRequestParams {
    r#ref: CompleteRequestRef,
    argument: CompleteRequestArgument,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum CompleteRequestRef {
    #[serde(rename = "ref/resource")]
    Resource { uri: String },
    #[serde(rename = "ref/prompt")]
    Prompt { name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompleteRequestArgument {
    name: String,
    value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompleteResult {
    values: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    total: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_more: Option<bool>,
}

// Roots

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListRootsRequestParams {
    #[serde(flatten)]
    request_base: RequestBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ListRootResult {
    roots: Vec<Root>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Root {
    uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RootsListChangedNotificationParams {
    #[serde(flatten)]
    notification_base: NotificationBaseParams,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "method", content = "params")]
pub enum RequestParams {
    Initialize(InitializeRequestParams),
    Ping(PingRequestParams),
    Paginated(Option<PaginatedRequestParams>),
    #[serde(rename = "resources/list")]
    ListResources(ListResourcesRequestParams),
    #[serde(rename = "resources/templates/list")]
    ListResourceTemplate(ListResourceTemplatesRequestParams),
    #[serde(rename = "resources/read")]
    ReadResource(ReadResourceRequestParams),
    #[serde(rename = "resources/subscribe")]
    Subscribe(SubscribeRequestParams),
    Unsubscribe(UnsubscribeRequestParams),
    #[serde(rename = "prompts/list")]
    ListPrompts(ListPromptsRequestParams),
    #[serde(rename = "prompts/get")]
    GetPrompt(GetPromptRequestParams),
    #[serde(rename = "tools/list")]
    ListTools(ListToolsRequestParams),
    #[serde(rename = "tools/call")]
    CallTool(CallToolRequestParams),
    #[serde(rename = "logging/setLevel")]
    SetLevel(SetLevelRequestParams),
    #[serde(rename = "sampling/createMessage")]
    CreateMessage(CreateMessageRequestParams),
    #[serde(rename = "completion/complete")]
    CompleteRequest(CompleteRequestParams),
    #[serde(rename = "roots/list")]
    ListRoots(ListRootsRequestParams),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "method")]
pub enum NotificationParams {
    #[serde(rename = "notifications/cancelled")]
    Cancelled(CancelledNotificationParams),
    #[serde(rename = "notifications/initialized")]
    Initialized(InitializedNotificationParams),
    #[serde(rename = "notifications/progress")]
    Progress(ProgressNotificationParams),
    #[serde(rename = "notifications/resources/list_changed")]
    ResourceListChanged(ResourceListChangedNotificationParams),
    #[serde(rename = "notifications/resources/updated")]
    ResourceUpdated(ResourceUpdatedNotificationParams),
    #[serde(rename = "notifications/prompts/list_changed")]
    PromptListChanged(PromptListChangedNotificationParams),
    #[serde(rename = "notifications/tools/list_changed")]
    ToolListChanged(ToolListChangedNotificationParams),
    #[serde(rename = "notifications/message")]
    LoggingMessage(LoggingMessageNotificationParams),
    #[serde(rename = "notifications/roots/list_changed")]
    RootsListChanged(RootsListChangedNotificationParams),
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase", untagged)]
pub enum ResultEnum {
    Empty(EmptyResult),
    Initialize(InitializeResult),
    Paginated(PaginatedResult),
    ListResources(ListResourcesResult),
    ListResourcesTemplate(ListResourcesTemplateResult),
    ReadResource(ReadResourceResult),
    ListPrompts(ListPromptsResult),
    GetPrompt(GetPromptResult),
    ListTools(ListToolsResult),
    CallTool(CallToolResult),
    CreateMessage(CreateMessageResult),
    Complete(CompleteResult),
    ListRoot(ListRootResult),
}

// Client Messages

// TODO Impl FROM<RequestParams> trait
// Messages that can only be received from clients
pub enum ClientRequestParams {
    Ping(PingRequestParams),
    Initialize(InitializeRequestParams),
    CompleteRequest(CompleteRequestParams),
    SetLevel(SetLevelRequestParams),
    GetPrompt(GetPromptRequestParams),
    ListPrompts(ListPromptsRequestParams),
    ListResources(ListResourcesRequestParams),
    ListResourceTemplate(ListResourceTemplatesRequestParams),
    ReadResource(ReadResourceRequestParams),
    Subscribe(SubscribeRequestParams),
    Unsubscribe(UnsubscribeRequestParams),
    CallTool(CallToolRequestParams),
    ListTools(ListToolsRequestParams),
}

impl From<RequestParams> for Option<ClientRequestParams> {
    fn from(value: RequestParams) -> Self {
        match value {
            RequestParams::Ping(x) => Some(ClientRequestParams::Ping(x)),
            RequestParams::Initialize(x) => Some(ClientRequestParams::Initialize(x)),
            RequestParams::CompleteRequest(x) => Some(ClientRequestParams::CompleteRequest(x)),
            RequestParams::SetLevel(x) => Some(ClientRequestParams::SetLevel(x)),
            RequestParams::GetPrompt(x) => Some(ClientRequestParams::GetPrompt(x)),
            RequestParams::ListPrompts(x) => Some(ClientRequestParams::ListPrompts(x)),
            RequestParams::ListResources(x) => Some(ClientRequestParams::ListResources(x)),
            RequestParams::ListResourceTemplate(x) => {
                Some(ClientRequestParams::ListResourceTemplate(x))
            }
            RequestParams::ReadResource(x) => Some(ClientRequestParams::ReadResource(x)),
            RequestParams::Subscribe(x) => Some(ClientRequestParams::Subscribe(x)),
            RequestParams::Unsubscribe(x) => Some(ClientRequestParams::Unsubscribe(x)),
            RequestParams::CallTool(x) => Some(ClientRequestParams::CallTool(x)),
            RequestParams::ListTools(x) => Some(ClientRequestParams::ListTools(x)),
            _ => None,
        }
    }
}

// TODO impl From trait
pub enum ClientNotificationParams {
    Cancelled(CancelledNotificationParams),
    Progress(ProgressNotificationParams),
    Initialized(InitializedNotificationParams),
    RootsListChanged(RootsListChangedNotificationParams),
}

impl From<NotificationParams> for Option<ClientNotificationParams> {
    fn from(value: NotificationParams) -> Self {
        match value {
            NotificationParams::Cancelled(x) => Some(ClientNotificationParams::Cancelled(x)),
            NotificationParams::Progress(x) => Some(ClientNotificationParams::Progress(x)),
            NotificationParams::Initialized(x) => Some(ClientNotificationParams::Initialized(x)),
            NotificationParams::RootsListChanged(x) => {
                Some(ClientNotificationParams::RootsListChanged(x))
            }
            _ => None,
        }
    }
}

// TODO impl From trait
pub enum ClientResult {
    Empty(EmptyResult),
    CreateMessage(CreateMessageResult),
    ListRoot(ListRootResult),
}

impl From<ResultEnum> for Option<ClientResult> {
    fn from(value: ResultEnum) -> Self {
        match value {
            ResultEnum::Empty(x) => Some(ClientResult::Empty(x)),
            ResultEnum::CreateMessage(x) => Some(ClientResult::CreateMessage(x)),
            ResultEnum::ListRoot(x) => Some(ClientResult::ListRoot(x)),
            _ => None,
        }
    }
}

// Server

pub enum ServerRequestParams {
    Ping(PingRequestParams),
    CreateMessage(CreateMessageRequestParams),
    ListRoots(ListRootsRequestParams),
}

impl From<RequestParams> for Option<ServerRequestParams> {
    fn from(value: RequestParams) -> Self {
        match value {
            RequestParams::Ping(x) => Some(ServerRequestParams::Ping(x)),
            RequestParams::CreateMessage(x) => Some(ServerRequestParams::CreateMessage(x)),
            RequestParams::ListRoots(x) => Some(ServerRequestParams::ListRoots(x)),
            _ => None,
        }
    }
}

pub enum ServerNotificationParams {
    Cancelled(CancelledNotificationParams),
    Progress(ProgressNotificationParams),
    LoggingMessage(LoggingMessageNotificationParams),
    ResourceUpdated(ResourceUpdatedNotificationParams),
    ResourceListChanged(ResourceListChangedNotificationParams),
    ToolListChanged(ToolListChangedNotificationParams),
    PromptListChanged(PromptListChangedNotificationParams),
}

impl From<NotificationParams> for Option<ServerNotificationParams> {
    fn from(value: NotificationParams) -> Self {
        match value {
            NotificationParams::Cancelled(x) => Some(ServerNotificationParams::Cancelled(x)),
            NotificationParams::Progress(x) => Some(ServerNotificationParams::Progress(x)),
            NotificationParams::LoggingMessage(x) => {
                Some(ServerNotificationParams::LoggingMessage(x))
            }
            NotificationParams::ResourceUpdated(x) => {
                Some(ServerNotificationParams::ResourceUpdated(x))
            }
            NotificationParams::ResourceListChanged(x) => {
                Some(ServerNotificationParams::ResourceListChanged(x))
            }
            NotificationParams::ToolListChanged(x) => {
                Some(ServerNotificationParams::ToolListChanged(x))
            }
            NotificationParams::PromptListChanged(x) => {
                Some(ServerNotificationParams::PromptListChanged(x))
            }
            _ => None,
        }
    }
}

pub enum ServerResult {
    Empty(EmptyResult),
    Initialize(InitializeResult),
    Complete(CompleteResult),
    GetPrompt(GetPromptResult),
    ListPrompts(ListPromptsResult),
    ListResources(ListResourcesResult),
    ListResourcesTemplate(ListResourcesTemplateResult),
    ReadResource(ReadResourceResult),
    CallTool(CallToolResult),
    ListTools(ListToolsResult),
}

impl From<ResultEnum> for Option<ServerResult> {
    fn from(value: ResultEnum) -> Self {
        match value {
            ResultEnum::Empty(x) => Some(ServerResult::Empty(x)),
            ResultEnum::Initialize(x) => Some(ServerResult::Initialize(x)),
            ResultEnum::Complete(x) => Some(ServerResult::Complete(x)),
            ResultEnum::GetPrompt(x) => Some(ServerResult::GetPrompt(x)),
            ResultEnum::ListPrompts(x) => Some(ServerResult::ListPrompts(x)),
            ResultEnum::ListResources(x) => Some(ServerResult::ListResources(x)),
            ResultEnum::ListResourcesTemplate(x) => Some(ServerResult::ListResourcesTemplate(x)),
            ResultEnum::ReadResource(x) => Some(ServerResult::ReadResource(x)),
            ResultEnum::CallTool(x) => Some(ServerResult::CallTool(x)),
            ResultEnum::ListTools(x) => Some(ServerResult::ListTools(x)),
            _ => None,
        }
    }
}
