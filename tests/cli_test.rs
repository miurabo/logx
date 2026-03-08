use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn help_displays_usage() {
    cargo_bin_cmd!("logx")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Zero-config log investigation CLI tool",
        ));
}

#[test]
fn version_displays_version() {
    cargo_bin_cmd!("logx")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("logx"));
}

#[test]
fn scan_subcommand_exists() {
    cargo_bin_cmd!("logx")
        .args(["scan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Scan log files"));
}

#[test]
fn errors_subcommand_exists() {
    cargo_bin_cmd!("logx")
        .args(["errors", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Extract errors"));
}

#[test]
fn filter_subcommand_exists() {
    cargo_bin_cmd!("logx")
        .args(["filter", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Filter log entries"));
}

#[test]
fn no_subcommand_shows_help() {
    cargo_bin_cmd!("logx")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}
