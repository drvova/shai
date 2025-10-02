use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool: String,
    #[serde(default)]
    pub args: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_stream: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speech: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviousCall {
    pub call: ToolCall,
    pub result: ToolCallResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMessage {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attached_files: Option<HashMap<String, String>>, // { filename: base64file, ... }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssistantMessage {
    pub assistant: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    User(UserMessage),
    Assistant(AssistantMessage),
    PreviousCall(PreviousCall),
}

// Note: AgentTool type needs to be imported from shai_core or defined
// For now, using a placeholder - we'll need to see what the actual tool type is
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTool {
    // Placeholder - adjust based on actual shai_core tool definition
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalQuery {
    pub model: String,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages: Option<Vec<Message>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AgentTool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalStreamingResponse {
    pub id: String,
    pub model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assistant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<ToolCall>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<ToolCallResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseMessage {
    Assistant(AssistantMessage),
    PreviousCall(PreviousCall),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiModalResponse {
    pub id: String,
    pub model: String,
    pub result: Vec<ResponseMessage>,
}