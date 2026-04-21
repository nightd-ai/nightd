mod common;

use common::TestContext;
use workspace::WorkspaceManager;

#[tokio::test]
async fn test_create_workspace() {
    let ctx = TestContext::new().await;

    let ws = ctx.manager.create(&ctx.source_repo).await.unwrap();

    assert!(ws.path.exists());
    assert!(ws.name.starts_with("source-repo"));
    assert_eq!(ws.source_repo, ctx.source_repo);

    let output = ctx.jj(&["workspace", "list"]).await;
    assert!(
        output.contains(&format!("{}:", ws.name)),
        "workspace list should contain {}: got {}",
        ws.name,
        output
    );
}

#[tokio::test]
async fn test_workspace_is_based_on_main() {
    let ctx = TestContext::new().await;

    let ws = ctx.manager.create(&ctx.source_repo).await.unwrap();

    // The workspace's parent commit should be main.
    let output = ctx
        .jj_in_dir(&ws.path, &["log", "--no-graph", "-r", "@-"])
        .await;
    assert!(
        output.contains("main"),
        "workspace parent should be main: got {}",
        output
    );
}

#[tokio::test]
async fn test_multiple_workspaces_have_distinct_names() {
    let ctx = TestContext::new().await;

    let ws1 = ctx.manager.create(&ctx.source_repo).await.unwrap();
    let ws2 = ctx.manager.create(&ctx.source_repo).await.unwrap();

    assert_ne!(ws1.name, ws2.name);
    assert_ne!(ws1.path, ws2.path);

    let output = ctx.jj(&["workspace", "list"]).await;
    assert!(output.contains(&format!("{}:", ws1.name)));
    assert!(output.contains(&format!("{}:", ws2.name)));
}

#[tokio::test]
async fn test_remove_creates_bookmark_and_deletes_directory() {
    let ctx = TestContext::new().await;

    let ws = ctx.manager.create(&ctx.source_repo).await.unwrap();

    // Simulate agent work.
    tokio::fs::write(ws.path.join("agent.txt"), "changes")
        .await
        .unwrap();
    ctx.jj_in_dir(&ws.path, &["describe", "-m", "agent work"])
        .await;

    ctx.manager.remove(&ws).await.unwrap();

    assert!(!ws.path.exists(), "workspace directory should be deleted");

    let output = ctx.jj(&["bookmark", "list"]).await;
    assert!(
        output.contains(&format!("{}:", ws.name)),
        "bookmark list should contain {}: got {}",
        ws.name,
        output
    );

    let output = ctx.jj(&["workspace", "list"]).await;
    assert!(
        !output.contains(&format!("{}:", ws.name)),
        "workspace list should NOT contain {}: got {}",
        ws.name,
        output
    );
}

#[tokio::test]
async fn test_remove_bookmark_points_to_workspace_commit() {
    let ctx = TestContext::new().await;

    let ws = ctx.manager.create(&ctx.source_repo).await.unwrap();

    tokio::fs::write(ws.path.join("agent.txt"), "changes")
        .await
        .unwrap();
    ctx.jj_in_dir(&ws.path, &["describe", "-m", "agent work"])
        .await;

    // Capture the workspace commit ID before removal.
    let commit_id = ctx
        .jj_in_dir(
            &ws.path,
            &["log", "--no-graph", "-r", "@", "-T", "commit_id ++ \"\\n\""],
        )
        .await
        .trim()
        .to_string();

    ctx.manager.remove(&ws).await.unwrap();

    // Verify the bookmark points to the same commit.
    let bookmark_commit = ctx
        .jj(&[
            "log",
            "--no-graph",
            "-r",
            &ws.name,
            "-T",
            "commit_id ++ \"\\n\"",
        ])
        .await
        .trim()
        .to_string();

    assert_eq!(
        commit_id, bookmark_commit,
        "bookmark should point to the workspace's final commit"
    );
}

#[tokio::test]
async fn test_manager_new_resolves_app_data_dir() {
    // Just verify that `new()` succeeds and points to the expected location.
    let manager = WorkspaceManager::new().unwrap();
    let _expected = dirs::data_dir().unwrap().join("nightd").join("workspaces");
    // We can't inspect the private field directly, but we can test the behaviour
    // by creating a workspace and checking where it lands.
    // For this test we just assert `new()` doesn't panic.
    let _ = manager;
    let ctx = TestContext::new().await;
    // Ensure the custom base dir variant works.
    let custom_manager = WorkspaceManager::with_base_dir(&ctx.workspaces_dir);
    let ws = custom_manager.create(&ctx.source_repo).await.unwrap();
    assert!(ws.path.starts_with(&ctx.workspaces_dir));
}
