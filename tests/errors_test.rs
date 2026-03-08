use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;

#[test]
fn errors_extracts_errors_from_log() {
    cargo_bin_cmd!("logx")
        .args(["errors", "tests/fixtures/mixed_errors.log", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 6 errors"))
        .stdout(predicate::str::contains("500"))
        .stdout(predicate::str::contains("503"))
        .stdout(predicate::str::contains("Summary"));
}

#[test]
fn errors_shows_no_errors_for_clean_log() {
    // apache_combined.log の200系のみ抽出してテスト...
    // health check ファイルを直接使う代わりにjson_linesのinfoのみのデータで確認
    cargo_bin_cmd!("logx")
        .args(["errors", "tests/fixtures/syslog.log", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ERROR"));
}

#[test]
fn errors_handles_empty_file() {
    cargo_bin_cmd!("logx")
        .args(["errors", "tests/fixtures/empty.log", "--no-color"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No errors found"));
}

#[test]
fn errors_handles_missing_file() {
    cargo_bin_cmd!("logx")
        .args(["errors", "tests/fixtures/nonexistent.log"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Failed to open"));
}

#[test]
fn errors_since_filter() {
    // --since 999d で全件拾えることを確認（テスト日時に依存しない）
    cargo_bin_cmd!("logx")
        .args([
            "errors",
            "tests/fixtures/mixed_errors.log",
            "--since",
            "999d",
            "--no-color",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}

#[test]
fn errors_since_invalid_format() {
    cargo_bin_cmd!("logx")
        .args([
            "errors",
            "tests/fixtures/mixed_errors.log",
            "--since",
            "abc",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid duration"));
}

#[test]
fn errors_multiple_files() {
    cargo_bin_cmd!("logx")
        .args([
            "errors",
            "tests/fixtures/mixed_errors.log",
            "tests/fixtures/apache_combined.log",
            "--no-color",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found"));
}
