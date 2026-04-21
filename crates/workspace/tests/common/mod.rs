use std::path::{Path, PathBuf};
use tempfile::TempDir;
use tokio::process::Command;
use workspace::WorkspaceManager;

#[allow(dead_code)]
pub struct TestContext {
    pub temp_dir: TempDir,
    pub source_repo: PathBuf,
    pub workspaces_dir: PathBuf,
    pub manager: WorkspaceManager,
}

impl TestContext {
    pub async fn new() -> Self {
        let temp_dir = tempfile::tempdir().unwrap();
        let source_repo = temp_dir.path().join("source-repo");
        let workspaces_dir = temp_dir.path().join("workspaces");

        std::fs::create_dir(&source_repo).unwrap();

        // Initialize a non-colocated jj repo with a main bookmark.
        run_jj(&["git", "init"], &source_repo).await;
        tokio::fs::write(source_repo.join("README.md"), "# test\n")
            .await
            .unwrap();
        run_jj(&["bookmark", "create", "main"], &source_repo).await;
        run_jj(&["describe", "-m", "initial"], &source_repo).await;

        let manager = WorkspaceManager::with_base_dir(&workspaces_dir);

        Self {
            temp_dir,
            source_repo,
            workspaces_dir,
            manager,
        }
    }

    pub async fn jj(&self, args: &[&str]) -> String {
        run_jj(args, &self.source_repo).await
    }

    pub async fn jj_in_dir(&self, dir: &Path, args: &[&str]) -> String {
        run_jj(args, dir).await
    }
}

async fn run_jj(args: &[&str], cwd: &Path) -> String {
    let output = Command::new("jj")
        .args(args)
        .current_dir(cwd)
        .output()
        .await
        .unwrap_or_else(|e| panic!("failed to run jj in {}: {}", cwd.display(), e));

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        panic!(
            "jj command failed in {}: jj {}\nstdout: {}\nstderr: {}",
            cwd.display(),
            args.join(" "),
            stdout,
            stderr
        );
    }

    stdout
}
