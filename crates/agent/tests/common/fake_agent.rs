//! Fake agent implementations for testing.
//!
//! This module provides fake agent components that can be used in tests
//! without spawning external processes.

#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::Arc;

use sacp::{AcpAgentToClientRequest, PromptResponse, StopReason};
use serde_json::Value;

use agent::Component;

/// Response type for fake agents.
pub type AgentResponse = Result<Value, String>;

/// Handler function type for dynamic responses.
pub type ResponseHandler = Arc<dyn Fn(&AcpAgentToClientRequest) -> AgentResponse + Send + Sync>;

/// Fake agent for testing that can be configured with custom responses.
///
/// # Examples
///
/// ```rust
/// use agent::Scheduler;
/// use fake_agent::{FakeAgent, ResponseBuilder};
///
/// // Create a fake agent with predefined responses
/// let agent = FakeAgent::new()
///     .with_response("WriteTextFileRequest", Ok(json!({"success": true})));
/// ```
pub struct FakeAgent {
    /// Predefined responses for specific request methods.
    responses: HashMap<String, AgentResponse>,
    /// Optional handler for dynamic responses.
    handler: Option<ResponseHandler>,
    /// Default response when no specific response is found.
    default_response: AgentResponse,
    /// Whether to echo back the request data in responses.
    echo_mode: bool,
}

impl std::fmt::Debug for FakeAgent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FakeAgent")
            .field("responses", &self.responses)
            .field("has_handler", &self.handler.is_some())
            .field("default_response", &self.default_response)
            .field("echo_mode", &self.echo_mode)
            .finish()
    }
}

impl FakeAgent {
    /// Create a new fake agent with default settings.
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
            handler: None,
            default_response: Err("No response configured".to_string()),
            echo_mode: false,
        }
    }

    /// Create a simple echo agent that echoes back request data.
    pub fn echo() -> Self {
        Self {
            responses: HashMap::new(),
            handler: None,
            default_response: Ok(serde_json::json!(null)),
            echo_mode: true,
        }
    }

    /// Set a predefined response for a specific request type.
    pub fn with_response(mut self, method: impl Into<String>, response: AgentResponse) -> Self {
        self.responses.insert(method.into(), response);
        self
    }

    /// Set multiple predefined responses.
    pub fn with_responses(
        mut self,
        responses: impl IntoIterator<Item = (impl Into<String>, AgentResponse)>,
    ) -> Self {
        for (method, response) in responses {
            self.responses.insert(method.into(), response);
        }
        self
    }

    /// Set a dynamic response handler.
    pub fn with_handler(
        mut self,
        handler: impl Fn(&AcpAgentToClientRequest) -> AgentResponse + Send + Sync + 'static,
    ) -> Self {
        self.handler = Some(Arc::new(handler));
        self
    }

    /// Set the default response for unhandled requests.
    pub fn with_default_response(mut self, response: AgentResponse) -> Self {
        self.default_response = response;
        self
    }

    /// Handle a request and generate the appropriate response.
    fn handle(&self, request: &AcpAgentToClientRequest) -> AgentResponse {
        // First, try the dynamic handler if set
        if let Some(handler) = &self.handler {
            return handler(request);
        }

        // Check for echo mode
        if self.echo_mode {
            let method = self.request_method_name(request);
            return Ok(serde_json::json!({
                "echo": true,
                "method": method,
            }));
        }

        // Try to find a predefined response based on request type
        let method = self.request_method_name(request);

        self.responses
            .get(&method)
            .cloned()
            .unwrap_or_else(|| self.default_response.clone())
    }

    /// Get the method name for a request type.
    fn request_method_name(&self, request: &AcpAgentToClientRequest) -> String {
        match request {
            AcpAgentToClientRequest::WriteTextFileRequest(_) => "WriteTextFileRequest",
            AcpAgentToClientRequest::ReadTextFileRequest(_) => "ReadTextFileRequest",
            AcpAgentToClientRequest::RequestPermissionRequest(_) => "RequestPermissionRequest",
            AcpAgentToClientRequest::CreateTerminalRequest(_) => "CreateTerminalRequest",
            AcpAgentToClientRequest::TerminalOutputRequest(_) => "TerminalOutputRequest",
            AcpAgentToClientRequest::ReleaseTerminalRequest(_) => "ReleaseTerminalRequest",
            AcpAgentToClientRequest::WaitForTerminalExitRequest(_) => "WaitForTerminalExitRequest",
            AcpAgentToClientRequest::KillTerminalCommandRequest(_) => "KillTerminalCommandRequest",
            AcpAgentToClientRequest::ExtMethodRequest(_) => "ExtMethodRequest",
            #[allow(unreachable_patterns)]
            _ => "unknown",
        }
        .to_string()
    }
}

impl Default for FakeAgent {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for FakeAgent {
    fn handle_request(&self, request: AcpAgentToClientRequest) -> agent::Result<Value> {
        match self.handle(&request) {
            Ok(value) => Ok(value),
            Err(msg) => Err(agent::Error::Acp(msg)),
        }
    }
}

/// Builder for constructing fake agent responses.
pub struct ResponseBuilder;

impl ResponseBuilder {
    /// Create a successful prompt response with text content.
    ///
    /// Note: PromptResponse only contains stop_reason and meta fields in the current ACP spec.
    pub fn prompt_response(_text: impl Into<String>) -> Value {
        let response = PromptResponse {
            stop_reason: StopReason::EndTurn,
            meta: None,
        };

        serde_json::to_value(response).unwrap()
    }

    /// Create a successful initialization response.
    pub fn init_response() -> Value {
        serde_json::json!({
            "protocol_version": "0.1",
            "agent_capabilities": {},
            "agent_info": null,
        })
    }

    /// Create a successful new session response.
    pub fn new_session_response(session_id: impl Into<String>) -> Value {
        serde_json::json!({
            "session_id": session_id.into(),
        })
    }

    /// Create an error response.
    pub fn error(message: impl Into<String>) -> AgentResponse {
        Err(message.into())
    }

    /// Create a success response with arbitrary JSON.
    pub fn success(value: Value) -> AgentResponse {
        Ok(value)
    }

    /// Create a file write success response.
    pub fn file_write_success() -> Value {
        serde_json::json!({ "success": true })
    }

    /// Create a file read response with content.
    pub fn file_read_response(content: impl Into<String>) -> Value {
        serde_json::json!({ "content": content.into() })
    }

    /// Create a terminal creation response.
    pub fn terminal_response(terminal_id: impl Into<String>) -> Value {
        serde_json::json!({ "terminal_id": terminal_id.into() })
    }

    /// Create a permission request response (approved).
    pub fn permission_approved() -> Value {
        serde_json::json!({ "outcome": "approved" })
    }

    /// Create a permission request response (denied).
    pub fn permission_denied(reason: impl Into<String>) -> Value {
        serde_json::json!({
            "outcome": "denied",
            "reason": reason.into(),
        })
    }
}

/// Predefined fake agents for common test scenarios.
pub mod predefined {
    use super::*;

    /// A fake agent that always returns success responses.
    pub fn always_success() -> FakeAgent {
        FakeAgent::new()
            .with_response(
                "WriteTextFileRequest",
                Ok(ResponseBuilder::file_write_success()),
            )
            .with_response(
                "ReadTextFileRequest",
                Ok(ResponseBuilder::file_read_response("test content")),
            )
            .with_response(
                "RequestPermissionRequest",
                Ok(ResponseBuilder::permission_approved()),
            )
            .with_response(
                "CreateTerminalRequest",
                Ok(ResponseBuilder::terminal_response("term-1")),
            )
            .with_default_response(Ok(serde_json::json!({"success": true})))
    }

    /// A fake agent that always returns errors.
    #[allow(dead_code)]
    pub fn always_error() -> FakeAgent {
        FakeAgent::new().with_default_response(ResponseBuilder::error("Simulated agent error"))
    }

    /// A fake agent that echoes back request method names.
    #[allow(dead_code)]
    pub fn echo() -> FakeAgent {
        FakeAgent::echo()
    }

    /// A fake agent with delayed responses (for testing timeouts/cancellation).
    #[allow(dead_code)]
    pub fn slow_responder() -> FakeAgent {
        FakeAgent::new().with_handler(|request| {
            // Simulate slow processing
            std::thread::sleep(std::time::Duration::from_millis(500));

            match request {
                AcpAgentToClientRequest::ReadTextFileRequest(_) => {
                    Ok(ResponseBuilder::file_read_response("slow response"))
                }
                _ => Ok(serde_json::json!({"success": true})),
            }
        })
    }
}
