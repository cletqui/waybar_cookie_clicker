use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const BIN: &str = env!("CARGO_BIN_EXE_waybar_cookie_clicker");

fn run(state: &Path, args: &[&str]) -> String {
    let out = Command::new(BIN)
        .arg("--state")
        .arg(state)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

fn json(s: &str) -> serde_json::Value {
    serde_json::from_str(s.trim()).unwrap()
}

#[test]
fn click_buy_reset_flow_persists_between_invocations() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("s.json");

    assert_eq!(json(&run(&state, &["show"]))["text"], "󰆘");
    for _ in 0..15 {
        run(&state, &["click"]);
    }
    assert_eq!(json(&run(&state, &["show"]))["text"], "󰆘 15");
    assert_eq!(json(&run(&state, &["slot", "0"]))["class"][1], "affordable");

    run(&state, &["buy-slot", "0"]);
    assert_eq!(json(&run(&state, &["show"]))["text"], "󰆘");
    assert!(
        json(&run(&state, &["show"]))["tooltip"]
            .as_str()
            .unwrap()
            .contains("Cursor x1")
    );

    run(&state, &["reset"]);
    assert!(
        !json(&run(&state, &["show"]))["tooltip"]
            .as_str()
            .unwrap()
            .contains("Cursor")
    );
}

#[test]
fn slot_and_show_do_not_write_the_save() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("s.json");
    run(&state, &["show"]);
    run(&state, &["slot", "0"]);
    assert!(!state.exists());
}

#[test]
fn config_command_emits_valid_module_json() {
    let dir = tempfile::tempdir().unwrap();
    let out = run(
        &dir.path().join("s.json"),
        &[
            "config",
            "--direction",
            "left",
            "--slots",
            "2",
            "--mode",
            "poll",
            "--signal",
            "11",
        ],
    );
    let v = json(&out);
    assert_eq!(
        v["group/group-cookie"]["drawer"]["transition-left-to-right"],
        false
    );
    assert_eq!(v["custom/cookie-s1"]["signal"], 11);
    assert!(v.get("custom/cookie-s2").is_none());
}

#[test]
fn watch_rejects_commands_without_output_view() {
    let dir = tempfile::tempdir().unwrap();
    let status = Command::new(BIN)
        .arg("--state")
        .arg(dir.path().join("s.json"))
        .args(["--watch", "click"])
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
}

#[test]
fn watch_prints_first_line_then_follows_state_changes() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("s.json");
    let mut child = Command::new(BIN)
        .arg("--state")
        .arg(&state)
        .args(["--watch", "--every", "60", "show"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    assert_eq!(json(&lines.next().unwrap().unwrap())["text"], "󰆘");
    run(&state, &["click"]);
    assert_eq!(json(&lines.next().unwrap().unwrap())["text"], "󰆘 1");
    child.kill().unwrap();
    child.wait().unwrap();
}

#[test]
fn watch_exits_when_its_parent_dies() {
    let dir = tempfile::tempdir().unwrap();
    let state = dir.path().join("s.json");
    let script = format!(
        "{BIN} --state {} --watch show >/dev/null 2>&1 & echo $!; sleep 0.5",
        state.display()
    );
    let out = Command::new("sh").arg("-c").arg(script).output().unwrap();
    let pid = String::from_utf8(out.stdout).unwrap().trim().to_string();
    let proc = Path::new("/proc").join(&pid);
    let deadline = Instant::now() + Duration::from_secs(5);
    while proc.exists() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    let alive = std::fs::read_to_string(proc.join("stat"))
        .map(|s| !s.contains(") Z"))
        .unwrap_or(false);
    assert!(!alive, "watcher {pid} outlived its parent");
}

#[test]
fn watch_exits_when_its_output_pipe_closes() {
    let dir = tempfile::tempdir().unwrap();
    let mut child = Command::new(BIN)
        .arg("--state")
        .arg(dir.path().join("s.json"))
        .args(["--watch", "--every", "60", "show"])
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
    lines.next().unwrap().unwrap();
    drop(lines);
    let deadline = Instant::now() + Duration::from_secs(5);
    while child.try_wait().unwrap().is_none() && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    let exited = child.try_wait().unwrap().is_some();
    if !exited {
        child.kill().ok();
    }
    assert!(exited, "watcher kept running after its stdout closed");
}
