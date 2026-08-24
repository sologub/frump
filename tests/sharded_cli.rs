use std::{fs, process::Command, time::{SystemTime, UNIX_EPOCH}};

fn fixture_root() -> std::path::PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("frump-sharded-cli-{}-{unique}", std::process::id()));
    fs::create_dir_all(root.join("frump/tasks")).unwrap();
    fs::write(root.join("frump/general.md"), "# Check\n\n## Team\n").unwrap();
    fs::write(root.join("frump/tasks/1.md"), "### Task 1 - First\n\nStatus: todo\n").unwrap();
    fs::write(root.join("frump/tasks/2.md"), "### Task 2 - Second\n\nStatus: todo\n").unwrap();
    root
}

#[test]
fn cli_updates_only_the_changed_sharded_task() {
    let root = fixture_root();
    let board = root.join("frump");
    let untouched = fs::read(root.join("frump/tasks/2.md")).unwrap();
    let binary = env!("CARGO_BIN_EXE_frump");

    let set = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "set", "1", "Status", "done"])
        .output()
        .unwrap();
    assert!(set.status.success(), "{}", String::from_utf8_lossy(&set.stderr));
    assert!(String::from_utf8(fs::read(root.join("frump/tasks/1.md")).unwrap()).unwrap().contains("Status: done"));
    assert_eq!(fs::read(root.join("frump/tasks/2.md")).unwrap(), untouched);

    let list = Command::new(binary)
        .args(["--file", board.to_str().unwrap(), "list"])
        .output()
        .unwrap();
    assert!(list.status.success());
    assert!(String::from_utf8(list.stdout).unwrap().contains("Task 1 - First"));
    let _ = fs::remove_dir_all(root);
}
