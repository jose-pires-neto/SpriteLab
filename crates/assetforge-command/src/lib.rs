//! Transport-independent contracts. No pixel, UI or MCP dependencies.
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const PROTOCOL_VERSION: u32 = 1;
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub request_id: Option<String>,
    pub document_id: Option<String>,
    pub expected_revision: Option<u64>,
    pub command: String,
    #[serde(default = "empty_args")]
    pub args: Value,
}
fn empty_args() -> Value {
    serde_json::json!({})
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommandError {
    pub code: String,
    pub message: String,
}
impl CommandError {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}
pub type Result<T> = std::result::Result<T, CommandError>;
#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub protocol_version: u32,
    pub request_id: Option<String>,
    pub ok: bool,
    pub revision: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<CommandError>,
}
impl Response {
    pub fn from_result(
        request_id: Option<String>,
        revision: Option<u64>,
        result: Result<Value>,
    ) -> Self {
        let (ok, result, error) = match result {
            Ok(v) => (true, Some(v), None),
            Err(e) => (false, None, Some(e)),
        };
        Self {
            protocol_version: PROTOCOL_VERSION,
            request_id,
            ok,
            revision,
            result,
            error,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Descriptor {
    pub id: String,
    pub summary: String,
    pub effect: String,
    pub undoable: bool,
    pub arguments_schema: Value,
}
pub fn decode<T: serde::de::DeserializeOwned>(v: Value) -> Result<T> {
    serde_json::from_value(v).map_err(|e| CommandError::new("invalid_arguments", e.to_string()))
}
