pub mod error;

pub use error::{Error, Result};

use std::path::Path;

pub struct InitResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub async fn run(command: &str, cwd: &Path) -> Result<InitResult> {
    let _ = command;
    let _ = cwd;
    todo!()
}
