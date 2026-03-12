use std::process::Stdio;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum AcpError {
    #[error("Failed to spawn ACP client: {0}")]
    SpawnError(String),

    #[error("ACP execution failed: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    IoError(String),
}

pub(crate) type Result<T> = std::result::Result<T, AcpError>;

#[derive(Clone)]
pub(crate) struct AcpClient;

impl AcpClient {
    pub(crate) fn new() -> Result<Self> {
        Ok(Self)
    }

    pub(crate) async fn execute_prompt(&self, prompt: &str) -> Result<String> {
        let mut child = tokio::process::Command::new("opencode")
            .arg("acp")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| AcpError::SpawnError(e.to_string()))?;

        use tokio::io::AsyncWriteExt;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .map_err(|e| AcpError::IoError(e.to_string()))?;
            drop(stdin);
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| AcpError::ExecutionError(e.to_string()))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(AcpError::ExecutionError(format!(
                "Process exited with code {:?}: {}",
                output.status.code(),
                stderr
            )));
        }

        String::from_utf8(output.stdout)
            .map_err(|e| AcpError::ExecutionError(format!("Invalid UTF-8: {}", e)))
    }
}

#[cfg(test)]
mod mock {
    use std::collections::HashMap;

    use super::Result;

    pub(crate) struct MockAcpClient {
        responses: HashMap<String, Result<String>>,
    }

    impl MockAcpClient {
        pub(crate) fn new() -> Self {
            Self {
                responses: HashMap::new(),
            }
        }

        pub(crate) fn with_response(mut self, prompt: &str, response: Result<String>) -> Self {
            self.responses.insert(prompt.to_string(), response);
            self
        }

        pub(crate) async fn execute_prompt(&self, prompt: &str) -> Result<String> {
            self.responses
                .get(prompt)
                .cloned()
                .unwrap_or(Ok(format!("Mock response for: {}", prompt)))
        }
    }
}

#[cfg(test)]
pub(crate) use mock::MockAcpClient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acp_client_new() {
        let client = AcpClient::new();
        assert!(client.is_ok());

        let _client = client.unwrap();
    }

    #[test]
    fn test_acp_error_display() {
        let spawn_err = AcpError::SpawnError("test error".to_string());
        assert!(spawn_err.to_string().contains("Failed to spawn ACP client"));

        let exec_err = AcpError::ExecutionError("execution failed".to_string());
        assert!(exec_err.to_string().contains("ACP execution failed"));
    }

    #[tokio::test]
    async fn test_mock_acp_client() {
        let client = MockAcpClient::new().with_response("hello", Ok("world".to_string()));

        let result = client.execute_prompt("hello").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "world");
    }

    #[tokio::test]
    async fn test_mock_acp_client_default_response() {
        let client = MockAcpClient::new();

        let result = client.execute_prompt("unknown prompt").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Mock response"));
    }

    #[tokio::test]
    async fn test_mock_acp_client_error() {
        let client = MockAcpClient::new()
            .with_response("fail", Err(AcpError::ExecutionError("error".to_string())));

        let result = client.execute_prompt("fail").await;
        assert!(result.is_err());
    }
}
