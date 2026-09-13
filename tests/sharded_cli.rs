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
        "### Bug 2 - Second\n\nStatus: todo\n",
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
fn commit_after_migration_stages_the_removed_single_file() {
    let root = unique_root("migrate-commit");
    let source = single_file_board(&root);
    for args in [
        vec!["init", "-q"],
        vec!["config", "user.name", "Frump Test"],
        vec!["config", "user.email", "frump-test@example.invalid"],
        vec!["add", "frump.md"],
        vec!["commit", "-qm", "Initial board"],
    ] {
        assert!(Command::new("git")
            .current_dir(&root)
            .args(args)
            .status()
            .unwrap()
            .success());
    }
    let binary = env!("CARGO_BIN_EXE_frump");
    let migrated = Command::new(binary)
        .args(["--file", source.to_str().unwrap(), "migrate"])
        .output()
        .unwrap();
    assert!(
        migrated.status.success(),
        "{}",
        String::from_utf8_lossy(&migrated.stderr)
    );

    let board = root.join("frump");
    let committed = Command::new(binary)
        .current_dir(&root)
        .args([
            "--file",
            board.to_str().unwrap(),
            "commit",
            "-m",
            "Migrate board",
        ])
        .output()
        .unwrap();
    assert!(
        committed.status.success(),
        "{}",
        String::from_utf8_lossy(&committed.stderr)
    );
    let files = Command::new("git")
        .current_dir(&root)
        .args(["show", "--name-status", "--format=", "HEAD"])
        .output()
        .unwrap();
    let files = String::from_utf8(files.stdout).unwrap();
    assert!(files.contains("D\tfrump.md"));
    assert!(files.contains("A\tfrump/general.md"));
    assert!(files.contains("A\tfrump/tasks/1.md"));
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
fn ready_omits_completed_tasks_and_keeps_unblocked_work() {
    let root = unique_root("ready");
    let board = root.join("frump.md");
    fs::write(
        &board,
        "# Check\n\n## Tasks\n\n### Task 1 - Completed\n\nStatus: done\n\n### Bug 2 - Ready\n\nStatus: todo\n\n### Task 3 - Blocked\n\nStatus: todo\nDepends On: 2\n",
    )
    .unwrap();

    let ready = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args(["--file", board.to_str().unwrap(), "ready"])
        .output()
        .unwrap();
    assert!(
        ready.status.success(),
        "{}",
        String::from_utf8_lossy(&ready.stderr)
    );
    let output = String::from_utf8(ready.stdout).unwrap();
    assert!(output.contains("Bug 2 - Ready"));
    assert!(!output.contains("Task 1 - Completed"));
    assert!(!output.contains("Task 3 - Blocked"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn update_refuses_empty_body_and_requires_an_explicit_clear() {
    let root = unique_root("body-clear");
    let board = single_file_board(&root);
    let before = fs::read(&board).unwrap();
    let binary = env!("CARGO_BIN_EXE_frump");

    let empty = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "1",
            "--body",
            "   ",
        ])
        .output()
        .unwrap();
    assert!(!empty.status.success());
    assert!(String::from_utf8_lossy(&empty.stderr).contains("use --clear-body"));
    assert_eq!(fs::read(&board).unwrap(), before);

    let set = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "1",
            "--body",
            "A durable record",
        ])
        .output()
        .unwrap();
    assert!(set.status.success());
    assert!(fs::read_to_string(&board)
        .unwrap()
        .contains("A durable record"));

    let clear = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "1",
            "--clear-body",
        ])
        .output()
        .unwrap();
    assert!(clear.status.success());
    assert!(!fs::read_to_string(&board)
        .unwrap()
        .contains("A durable record"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn body_replacement_stays_a_replacement_and_appends_after_the_complete_body() {
    let root = unique_root("body-order");
    let board = root.join("frump.md");
    fs::write(
        &board,
        "# Check\n\n## Tasks\n\n### Investigation 4 - Preserve reports\n\nFraming text.\n\nINVESTIGATION REPORT: first report.\n\nStatus: investigations\n",
    )
    .unwrap();
    let binary = env!("CARGO_BIN_EXE_frump");

    for body in ["First replacement", "Second replacement"] {
        let update = Command::new(binary)
            .args([
                "--file",
                board.to_str().unwrap(),
                "update",
                "4",
                "--body",
                body,
            ])
            .output()
            .unwrap();
        assert!(
            update.status.success(),
            "{}",
            String::from_utf8_lossy(&update.stderr)
        );
        assert!(String::from_utf8_lossy(&update.stdout).contains("Replaced body for task 4"));
    }
    let replaced = fs::read_to_string(&board).unwrap();
    assert!(!replaced.contains("First replacement"));
    assert_eq!(replaced.matches("Second replacement").count(), 1);
    assert!(!replaced.contains("INVESTIGATION REPORT: first report."));

    let append = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "4",
            "--append-body",
            "Dated evidence",
        ])
        .output()
        .unwrap();
    assert!(
        append.status.success(),
        "{}",
        String::from_utf8_lossy(&append.stderr)
    );
    let appended = fs::read_to_string(&board).unwrap();
    assert!(appended.find("Second replacement").unwrap() < appended.find("_Update ").unwrap());
    assert!(appended.find("_Update ").unwrap() < appended.find("Dated evidence").unwrap());
    assert_eq!(appended.matches("Second replacement").count(), 1);
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

#[test]
fn sharded_next_accepts_mixed_card_types_and_refuses_missing_ids_atomically() {
    let root = fixture_root();
    let board = root.join("frump");
    let binary = env!("CARGO_BIN_EXE_frump");

    let set = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "next", "1", "2"])
        .output()
        .unwrap();
    assert!(
        set.status.success(),
        "{}",
        String::from_utf8_lossy(&set.stderr)
    );
    let listed = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "next"])
        .output()
        .unwrap();
    assert_eq!(String::from_utf8(listed.stdout).unwrap().trim(), "1\n2");

    let general = board.join("general.md");
    let before = fs::read(&general).unwrap();
    let missing = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "next", "1", "999"])
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("missing task 999"));
    assert_eq!(fs::read(general).unwrap(), before);
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

#[cfg(unix)]
#[test]
fn assignment_changes_are_announced_after_the_board_is_written() {
    use std::os::unix::fs::PermissionsExt;

    let root = unique_root("assignment-message");
    let board = root.join("frump.md");
    fs::write(&board, "# Check\n\n## Tasks\n").unwrap();
    let tools = root.join("tools");
    fs::create_dir(&tools).unwrap();
    let metateam = tools.join("metateam");
    fs::write(
        &metateam,
        "#!/bin/sh\n/usr/bin/grep -q '^Assigned To: ' \"$FRUMP_BOARD\" || exit 23\nprintf '%s\\n' \"$@\" >> \"$FRUMP_MESSAGE_ARGS\"\nprintf '\\n' >> \"$FRUMP_MESSAGE_ARGS\"\nprintf 'delivered by test\\n'\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&metateam).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&metateam, permissions).unwrap();
    let arguments = root.join("assignment-arguments");
    let binary = env!("CARGO_BIN_EXE_frump");

    let added = Command::new(binary)
        .args([
            "--file",
            board.to_str().unwrap(),
            "add",
            "Fixture",
            "--assignee",
            "Ada",
        ])
        .env("PATH", &tools)
        .env("FRUMP_MESSAGE_ARGS", &arguments)
        .env("FRUMP_BOARD", &board)
        .output()
        .unwrap();
    assert!(
        added.status.success(),
        "{}",
        String::from_utf8_lossy(&added.stderr)
    );
    let added_output = String::from_utf8(added.stdout).unwrap();
    let id = added_output
        .lines()
        .find(|line| line.starts_with("Added "))
        .unwrap()
        .split_whitespace()
        .nth(2)
        .unwrap()
        .to_string();

    for args in [
        vec![
            "--file",
            board.to_str().unwrap(),
            "update",
            &id,
            "--body",
            "replacement",
        ],
        vec!["--file", board.to_str().unwrap(), "assign", &id, "Ada"],
        vec!["--file", board.to_str().unwrap(), "assign", &id, "Bob"],
        vec![
            "--file",
            board.to_str().unwrap(),
            "set",
            &id,
            "Assigned To",
            "Carol",
        ],
    ] {
        let output = Command::new(binary)
            .args(args)
            .env("PATH", &tools)
            .env("FRUMP_MESSAGE_ARGS", &arguments)
            .env("FRUMP_BOARD", &board)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let messages = fs::read_to_string(arguments).unwrap();
    let invocations: Vec<Vec<_>> = messages
        .trim_end()
        .split("\n\n")
        .map(|invocation| invocation.lines().collect())
        .collect();
    assert_eq!(invocations.len(), 3);
    let first_body = format!("Task {id} is assigned to Ada.");
    assert_eq!(
        invocations[0],
        vec![
            "crew",
            "message",
            "--from",
            "frump",
            "all",
            first_body.as_str()
        ]
    );
    assert_eq!(invocations[1][5], format!("Task {id} is assigned to Bob."));
    assert_eq!(
        invocations[2][5],
        format!("Task {id} is assigned to Carol.")
    );
    assert!(!messages.contains("replacement"));
    assert_eq!(
        Command::new(binary)
            .args(["--file", board.to_str().unwrap(), "show", &id])
            .output()
            .unwrap()
            .stdout
            .windows("replacement".len())
            .filter(|window| *window == b"replacement")
            .count(),
        1
    );
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn append_body_msg_without_metateam_saves_the_update_and_succeeds() {
    let root = unique_root("append-message-missing");
    let board = single_file_board(&root);
    let empty_path = root.join("no-tools");
    fs::create_dir(&empty_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args([
            "--file",
            board.to_str().unwrap(),
            "update",
            "1",
            "--append-body-msg",
            "Evidence accepted without a notification channel",
        ])
        .env("PATH", &empty_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Messaged with metateam"),
        "metateam is absent, so nothing may be reported as messaged: {stdout}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "an absent optional command is not an error"
    );
    assert!(fs::read_to_string(&board)
        .unwrap()
        .contains("Evidence accepted without a notification channel"));
    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn assignment_without_metateam_keeps_the_assignee_and_succeeds() {
    let root = unique_root("assignment-missing");
    let board = single_file_board(&root);
    let empty_path = root.join("no-tools");
    fs::create_dir(&empty_path).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_frump"))
        .args(["--file", board.to_str().unwrap(), "assign", "1", "Ada"])
        .env("PATH", &empty_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains("Messaged with metateam"),
        "metateam is absent, so nothing may be reported as messaged: {stdout}"
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).is_empty(),
        "an absent optional command is not an error"
    );
    assert!(fs::read_to_string(&board)
        .unwrap()
        .contains("Assigned To: Ada"));
    let _ = fs::remove_dir_all(root);
}
