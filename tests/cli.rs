use std::path::PathBuf;
use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xyzw-petsim"))
        .args(args)
        .output()
        .expect("failed to run xyzw-petsim")
}

fn temporary_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "xyzw_petsim_cli_{}_{}_{}",
        std::process::id(),
        name,
        std::thread::current().name().unwrap_or("test")
    ))
}

#[test]
fn top_level_help_lists_simulation_modes() {
    let output = run(&["--help"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(output.status.success());
    assert!(stdout.contains("target-cost"));
    assert!(stdout.contains("stock-drain"));
}

#[test]
fn invalid_target_returns_an_error() {
    let output = run(&["target-cost", "-T", "1", "-N", "1", "-I", "-q"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert!(!output.status.success());
    assert!(stderr.contains("目标等级必须在 2 到 7 之间"));
}

#[test]
fn seeded_single_trial_is_reproducible() {
    let args = [
        "target-cost",
        "-T",
        "5",
        "-N",
        "1",
        "-S",
        "123",
        "-I",
        "-q",
        "-M",
        "none",
    ];

    let first = run(&args);
    let second = run(&args);

    assert!(first.status.success());
    assert!(second.status.success());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn target_cost_exports_csv_and_json() {
    let csv_path = temporary_path("samples.csv");
    let json_path = temporary_path("report.json");
    let output = run(&[
        "target-cost",
        "-T",
        "2",
        "-N",
        "3",
        "-t",
        "1",
        "-S",
        "123",
        "-I",
        "-q",
        "-M",
        "none",
        "-C",
        csv_path.to_str().unwrap(),
        "-J",
        json_path.to_str().unwrap(),
    ]);

    assert!(output.status.success());

    let csv = std::fs::read_to_string(&csv_path).unwrap();
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_path).unwrap()).unwrap();
    std::fs::remove_file(csv_path).unwrap();
    std::fs::remove_file(json_path).unwrap();

    assert_eq!(csv.lines().next(), Some("trial,eggs"));
    assert_eq!(csv.lines().count(), 4);
    assert_eq!(json["config"]["target"], 2);
    assert_eq!(json["result"]["trials"], 3);
    assert_eq!(json["theory"]["no_pity_exact"], serde_json::Value::Null);
}
