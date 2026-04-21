use crate::models::AgentConfig;
use sacp::ContentBlock;
use std::path::Path;

pub struct AcpClient;

impl AcpClient {
    pub async fn spawn(agent: &AgentConfig, model: Option<&str>) -> crate::Result<Self> {
        let _ = agent;
        let _ = model;
        todo!()
    }

    pub async fn run_session(
        &mut self,
        cwd: &Path,
        prompt: &str,
        on_update: impl Fn(ContentBlock),
    ) -> crate::Result<String> {
        let _ = cwd;
        let _ = prompt;
        let _ = on_update;
        todo!()
    }
}
