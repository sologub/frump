use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn fixture_root() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("frump-sharded-cli-{}-{unique}", std::process::id()));
    fs::create_dir_all(root.join("frump/tasks")).unwrap();
    fs::write(root.join("frump/general.md"), "# Check\n\n## Team\n").unwrap();
    fs::write(
        root.join("frump/tasks/1.md"),
        "### Task 1 - First\n\nStatus: todo\n",
    )
    .unwrap();
    fs::write(
        root.join("frump/tasks/2.md"),
        "### Task 2 - Second\n\nStatus: todo\n",
    )
    .unwrap();
    root
}

#[test]
fn cli_updates_only_the_changed_sharded_task() {
    let root = fixture_root();
    let board = root.join("frump");
    let untouched = fs::read(root.join("frump/tasks/2.md")).unwrap();
    let binary = env!("CARGO_BIN_EXE_frump");

    let set = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "set",
            "1",
            "Status",
            "done",
        ])
        .output()
        .unwrap();
    assert!(
        set.status.success(),
        "{}",
        String::from_utf8_lossy(&set.stderr)
    );
    assert!(
        String::from_utf8(fs::read(root.join("frump/tasks/1.md")).unwrap())
            .unwrap()
            .contains("Status: done")
    );
    assert_eq!(fs::read(root.join("frump/tasks/2.md")).unwrap(), untouched);
    assert!(!root.join("frump/.frump.lock").exists());

    let list = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(list.status.success());
    assert!(String::from_utf8(list.stdout)
        .unwrap()
        .contains("Task 1 - First"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn sharded_commit_tracks_task_data_without_a_lock_file() {
    let root = fixture_root();
    let board = root.join("frump");
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Frump Test"],
        vec!["config", "user.email", "frump-test@example.invalid"],
        vec!["add", "frump"],
        vec!["commit", "-qm", "Initial board"],
    ] {
        let status = Command::new("git")
            .current_dir(&root)
            .args(args)
            .status()
            .unwrap();
        assert!(status.success());
    }

    let binary = env!("CARGO_BIN_EXE_frump");
    let set = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "set",
            "1",
            "Status",
            "done",
        ])
        .output()
        .unwrap();
    assert!(
        set.status.success(),
        "{}",
        String::from_utf8_lossy(&set.stderr)
    );
    assert!(!board.join(".frump.lock").exists());

    let commit = Command::new(binary)
        .current_dir(&root)
        .args([
            "--file",
            board.to_str().unwrap(),
            "commit",
            "-m",
            "Update task",
        ])
        .output()
        .unwrap();
    assert!(
        commit.status.success(),
        "{}",
        String::from_utf8_lossy(&commit.stderr)
    );
    let files = Command::new("git")
        .current_dir(&root)
        .args(["show", "--name-only", "--format=", "HEAD"])
        .output()
        .unwrap();
    let files = String::from_utf8(files.stdout).unwrap();
    assert!(files.contains("frump/general.md"));
    assert!(files.contains("frump/tasks/1.md"));
    assert!(!files.contains(".frump.lock"));
    let _ = fs::remove_dir_all(root);
}

fn single_file_board(root: &std::path::Path) -> std::path::PathBuf {
    let board = root.join("frump.md");
    fs::write(
        &board,
        "# Check\n\n## Team\n\n## Tasks\n\n### Task 1 - First\n\nStatus: todo\nReview: approved\nLast Updated: 2026-08-25T10:00:00Z\n\n### Bug 2 - Second\n\nStatus: todo\nAssigned To: Ada\nEvidence Kind: test\nLast Updated: 2026-08-25T11:00:00Z\n\n### Task 3 - Third\n\nStatus: working\n\n",
    )
    .unwrap();
    board
}

fn unique_root(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("frump-{label}-{}-{unique}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    root
}

#[test]
fn migrate_replaces_the_source_with_a_verified_sharded_board() {
    let root = unique_root("migrate");
    let source = single_file_board(&root);
    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args(["--file", source.to_str().unwrap(), "migrate"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let sharded = root.join("frump");
    assert!(!source.exists());
    assert!(sharded.join("general.md").is_file());
    assert!(sharded.join("tasks/1.md").is_file());
    let list = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args([
            "--file",
            sharded.to_str().unwrap(),
            "list",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(list.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&list.stdout)
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        3
    );
    let _ = fs::remove_dir_all(root);
}

#[test]
fn migrate_refuses_an_existing_destination_without_touching_the_source() {
    let root = unique_root("migrate-refusal");
    let source = single_file_board(&root);
    let original = fs::read(&source).unwrap();
    fs::create_dir(root.join("frump")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args(["--file", source.to_str().unwrap(), "migrate"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert_eq!(fs::read(&source).unwrap(), original);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn list_filters_sorts_and_emits_machine_readable_tasks() {
    let root = unique_root("list");
    let board = single_file_board(&root);
    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args([
            "--file",
            board.to_str().unwrap(),
            "list",
            "--property",
            "Status=todo",
            "--missing",
            "Assigned To",
            "--sort",
            "last-updated",
            "--desc",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tasks: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(tasks.as_array().unwrap()[0]["id"], 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn next_blocks_out_of_order_todo_transitions_and_prunes_done_work() {
    let root = unique_root("next");
    let board = single_file_board(&root);
    let bin = env!("CARGO_BIN_EXE_frump");
    let set = Command::new(bin)
        .args(["--file", board.to_str().unwrap(), "next", "1", "2"])
        .output()
        .unwrap();
    assert!(set.status.success());
    let blocked = Command::new(bin)
        .args([
            "--file",
            board.to_str().unwrap(),
            "set",
            "2",
            "Status",
            "working",
        ])
        .output()
        .unwrap();
    assert!(!blocked.status.success());
    assert!(String::from_utf8_lossy(&blocked.stderr).contains("not the next planned task"));
    let unset_blocked = Command::new(bin)
        .args(["--file", board.to_str().unwrap(), "unset", "2", "Status"])
        .output()
        .unwrap();
    assert!(!unset_blocked.status.success());
    assert!(Command::new(bin)
        .args([
            "--file",
            board.to_str().unwrap(),
            "set",
            "1",
            "Status",
            "working"
        ])
        .status()
        .unwrap()
        .success());
    assert!(Command::new(bin)
        .args([
            "--file",
            board.to_str().unwrap(),
            "set",
            "1",
            "Status",
            "done"
        ])
        .status()
        .unwrap()
        .success());
    let next = Command::new(bin)
        .args(["--file", board.to_str().unwrap(), "next"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(next.stdout).unwrap().trim(), "2");
    assert!(Command::new(bin)
        .args(["--file", board.to_str().unwrap(), "next", "--clear"])
        .status()
        .unwrap()
        .success());
    let cleared = Command::new(bin)
        .args(["--file", board.to_str().unwrap(), "next"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8(cleared.stdout).unwrap().trim(),
        "No next tasks planned."
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn append_body_msg_saves_the_update_and_broadcasts_the_raw_fragment() {
    use std::os::unix::fs::PermissionsExt;

    let root = unique_root("append-message");
    let board = single_file_board(&root);
    let tools = root.join("tools");
    fs::create_dir(&tools).unwrap();
    let metateam = tools.join("metateam");
    fs::write(
        &metateam,
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"$FRUMP_MESSAGE_ARGS\"\nprintf 'delivered by test\\n'\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&metateam).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&metateam, permissions).unwrap();
    let arguments = root.join("message-arguments");

    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "1",
            "--append-body-msg",
            "Evidence accepted",
        ])
        .env("PATH", &tools)
        .env("FRUMP_MESSAGE_ARGS", &arguments)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout)
        .contains("Messaged with metateam: delivered by test"));
    assert_eq!(
        fs::read_to_string(arguments).unwrap(),
        "crew\nmessage\nall\nEvidence accepted\n"
    );
    assert!(fs::read_to_string(board)
        .unwrap()
        .contains("Evidence accepted"));
    let _ = fs::remove_dir_all(root);
}
