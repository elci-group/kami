use assert_cmd::Command;
use predicates::prelude::*;

fn kami() -> Command {
    Command::cargo_bin("kami").unwrap()
}

#[test]
fn init_creates_the_state_directory() {
    let dir = tempfile::tempdir().unwrap();
    kami()
        .args(["--path", dir.path().to_str().unwrap(), "init"])
        .assert()
        .success()
        .stdout(predicate::str::contains("initialized"));
    assert!(dir.path().join(".kami").is_dir());
}

#[test]
fn inspect_reports_real_measured_size() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}\n").unwrap();

    kami()
        .args(["--path", dir.path().to_str().unwrap(), "inspect"])
        .assert()
        .success()
        .stdout(predicate::str::contains("source LOC: 1"));
}

#[test]
fn status_lists_elci_adapters_as_json() {
    let dir = tempfile::tempdir().unwrap();
    let output = kami()
        .args([
            "--path",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "status",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();
    let adapters = value["adapters"].as_array().unwrap();
    assert!(adapters.iter().any(|a| a["name"] == "bart"));
}

#[test]
fn state_build_then_show_round_trips_the_mission() {
    let dir = tempfile::tempdir().unwrap();

    kami()
        .args([
            "--path",
            dir.path().to_str().unwrap(),
            "state",
            "build",
            "--mission",
            "make the project great",
        ])
        .assert()
        .success();

    kami()
        .args([
            "--path",
            dir.path().to_str().unwrap(),
            "state",
            "show",
            "--id",
            "0",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("make the project great"))
        .stdout(predicate::str::contains("STATE-000"));
}

#[test]
fn state_build_refuses_to_run_twice() {
    let dir = tempfile::tempdir().unwrap();

    kami()
        .args(["--path", dir.path().to_str().unwrap(), "state", "build"])
        .assert()
        .success();

    kami()
        .args(["--path", dir.path().to_str().unwrap(), "state", "build"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not implemented"));
}
