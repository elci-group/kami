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
fn state_build_folds_first_loop_battery_evidence_into_the_state() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src").join("main.rs"), "fn main() {}\n").unwrap();

    let output = kami()
        .args([
            "--path",
            dir.path().to_str().unwrap(),
            "--format",
            "json",
            "state",
            "build",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let value: serde_json::Value = serde_json::from_slice(&output).unwrap();

    let battery = value["battery"].as_array().unwrap();
    let tools: Vec<&str> = battery.iter().filter_map(|p| p["tool"].as_str()).collect();
    assert_eq!(
        tools,
        vec!["amber", "traci", "fract", "ferret", "wilder", "curly", "jeenome"]
    );

    // curly/jeenome are documented as not applicable rather than invoked.
    let curly = battery.iter().find(|p| p["tool"] == "curly").unwrap();
    assert_eq!(curly["outcome"], "not_applicable");

    // Every evidence id the battery recorded must actually be readable
    // back from the state's evidence_ids — no evidence produced and
    // then silently dropped.
    let state_evidence_ids: Vec<&str> = value["state"]["evidence_ids"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    for pass in battery {
        if let Some(id) = pass["evidence_id"].as_str() {
            assert!(
                state_evidence_ids.contains(&id),
                "evidence id {id} from {} missing from state",
                pass["tool"]
            );
        }
    }
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
