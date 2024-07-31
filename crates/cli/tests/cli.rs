use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn empty_call() {
    let mut cmd = Command::cargo_bin("staff-cli").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Commands"));
}

#[test]
fn grimoire() {
    // Add new grimoire with empty path or name
    let mut cmd = Command::cargo_bin("staff-cli").unwrap();
    cmd.args(["grimoires", "add"]).assert().failure();

    // Add new grimoire
    let mut cmd = Command::cargo_bin("staff-cli").unwrap();
    cmd.args(["grimoires", "add", "any text"])
        .assert()
        .success();

    // List of available grimoires
    let mut cmd = Command::cargo_bin("staff-cli").unwrap();
    cmd.args(["grimoires"]).assert().success();
}
