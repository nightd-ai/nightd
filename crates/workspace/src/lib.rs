pub mod error;

pub use error::{Error, Result};

use std::path::{Path, PathBuf};

pub struct Workspace {
    pub path: PathBuf,
    pub branch: String,
}

pub async fn create(repo_url: &str, branch: &str, base_dir: &Path) -> Result<Workspace> {
    let _ = repo_url;
    let _ = branch;
    let _ = base_dir;
    todo!()
}

pub async fn push_and_remove(ws: &Workspace) -> Result<()> {
    let _ = ws;
    todo!()
}
