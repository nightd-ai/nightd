pub mod error;

pub use error::{Error, Result};

use std::path::{Path, PathBuf};
use tokio::process::Command;
use tracing::{debug, instrument};

/// Represents a Jujutsu workspace created from a source repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    /// The unique name of the workspace (e.g. `myrepo-swift-river`).
    pub name: String,
    /// The absolute path where the workspace is located.
    pub path: PathBuf,
    /// The absolute path to the source repository this workspace was created from.
    pub source_repo: PathBuf,
}

/// Manages the lifecycle of Jujutsu workspaces.
#[derive(Debug, Clone)]
pub struct WorkspaceManager {
    workspaces_dir: PathBuf,
}

impl WorkspaceManager {
    /// Creates a manager that stores workspaces in the platform's app data
    /// directory under `nightd/workspaces`.
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir().ok_or(Error::NoAppDataDir)?;
        Ok(Self::with_base_dir(
            data_dir.join("nightd").join("workspaces"),
        ))
    }

    /// Creates a manager with a custom base directory for workspaces.
    pub fn with_base_dir(workspaces_dir: impl Into<PathBuf>) -> Self {
        Self {
            workspaces_dir: workspaces_dir.into(),
        }
    }

    /// Creates a new Jujutsu workspace from `source_repo`, basing it on the
    /// `main` bookmark. The workspace is created inside the manager's
    /// workspaces directory with a name like `{repo_name}-{petname}`.
    #[instrument(skip(self, source_repo))]
    pub async fn create(&self, source_repo: impl AsRef<Path>) -> Result<Workspace> {
        let source_repo = std::fs::canonicalize(source_repo.as_ref())?;

        if !source_repo.join(".jj").is_dir() {
            return Err(Error::InvalidSourceRepo(format!(
                "{} does not contain a .jj directory",
                source_repo.display()
            )));
        }

        let repo_name = source_repo
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("repo")
            .to_string();

        let name = self.generate_unique_name(&repo_name, &source_repo).await?;
        let path = self.workspaces_dir.join(&name);

        tokio::fs::create_dir_all(&self.workspaces_dir).await?;

        debug!(%name, path = %path.display(), "creating jj workspace");

        run_jj(
            &[
                "workspace",
                "add",
                "--name",
                &name,
                "-r",
                "main",
                path.as_os_str().to_str().unwrap(),
            ],
            &source_repo,
        )
        .await?;

        Ok(Workspace {
            name,
            path,
            source_repo,
        })
    }

    /// Removes a workspace: forgets it in Jujutsu, deletes the directory, and
    /// creates a bookmark in the source repo named after the workspace pointing
    /// to the workspace's final commit.
    #[instrument(skip(self, workspace))]
    pub async fn remove(&self, workspace: &Workspace) -> Result<()> {
        debug!(name = %workspace.name, "removing workspace");

        // 1. Resolve the workspace's working-copy commit ID before we forget it.
        let commit_id = run_jj(
            &[
                "log",
                "--no-graph",
                "-r",
                &format!("{}@", workspace.name),
                "-T",
                "commit_id ++ \"\\n\"",
            ],
            &workspace.source_repo,
        )
        .await?
        .trim()
        .to_string();

        // 2. Forget the workspace in the source repo.
        run_jj(
            &["workspace", "forget", &workspace.name],
            &workspace.source_repo,
        )
        .await?;

        // 3. Delete the workspace directory.
        tokio::fs::remove_dir_all(&workspace.path).await?;

        // 4. Create a bookmark at the resolved commit.
        run_jj(
            &["bookmark", "create", &workspace.name, "-r", &commit_id],
            &workspace.source_repo,
        )
        .await?;

        Ok(())
    }

    /// Generates a unique workspace name, retrying on collision.
    async fn generate_unique_name(&self, repo_name: &str, source_repo: &Path) -> Result<String> {
        for _ in 0..10 {
            let suffix = petname::petname(2, "-").ok_or(Error::NameCollision)?;
            let name = format!("{}-{}", repo_name, suffix);
            let path = self.workspaces_dir.join(&name);

            if path.exists() {
                continue;
            }

            // Also check that a workspace with this name doesn't already exist
            // in the source repo.
            let list = run_jj(&["workspace", "list"], source_repo).await?;
            if list
                .lines()
                .any(|line| line.starts_with(&format!("{}:", name)))
            {
                continue;
            }

            return Ok(name);
        }

        Err(Error::NameCollision)
    }
}

impl Default for WorkspaceManager {
    fn default() -> Self {
        Self::new().expect("failed to create default workspace manager")
    }
}

/// Runs a `jj` command in the given working directory and returns stdout on
/// success.
async fn run_jj(args: &[&str], cwd: &Path) -> Result<String> {
    let mut cmd = Command::new("jj");
    cmd.args(args).current_dir(cwd);

    debug!(command = format!("jj {}", args.join(" ")), dir = %cwd.display(), "spawning jj");

    let output = cmd.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();

    if !output.status.success() {
        return Err(Error::JujutsuCommand {
            message: format!("jj exited with code {}", output.status),
            stdout,
            stderr,
        });
    }

    Ok(stdout)
}
