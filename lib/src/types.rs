use serde::{Deserialize, Serialize};

use crate::jsonrpc::JsonRpcResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct AddParams {
    pub a: i64,
    pub b: i64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsListParams {
    pub cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsListResult {
    pub tools: Vec<Tool>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCallParams {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolContent {
    #[serde(rename = "type")]
    pub content_type: String,
    #[serde(flatten)]
    pub content: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsCallResult {
    pub content: Vec<ToolContent>,
    pub is_error: bool,
}

/// Trait for implementing individual tools
pub trait ToolImplementation {
    /// Get the tool definition (name, description, schema)
    fn get_tool(&self) -> Tool;
    
    /// Execute the tool with the given arguments
    fn call(&self, arguments: serde_json::Value) -> ToolsCallResult;
}

/// Helper function to create text content
pub fn text_content(text: String) -> ToolContent {
    ToolContent {
        content_type: "text".to_string(),
        content: serde_json::json!({ "text": text }),
    }
}

/// Helper function to create error content
pub fn error_content(message: String) -> ToolsCallResult {
    ToolsCallResult {
        content: vec![text_content(message)],
        is_error: true,
    }
}

/// Helper function to create success content
pub fn success_content(content: Vec<ToolContent>) -> ToolsCallResult {
    ToolsCallResult {
        content,
        is_error: false,
    }
}
