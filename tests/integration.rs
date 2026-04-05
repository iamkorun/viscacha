use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn viscacha() -> Command {
    Command::cargo_bin("viscacha").unwrap()
}

#[test]
fn no_version_files_prints_message_and_exits_0() {
    let tmp = TempDir::new().unwrap();
    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No version constraint files found"));
}

#[test]
fn quiet_mode_no_output_on_empty_dir() {
    let tmp = TempDir::new().unwrap();
    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .arg("--quiet")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn detects_nvmrc_and_shows_node() {
    let tmp = TempDir::new().unwrap();
    // Use a version that almost certainly won't match to test table output
    fs::write(tmp.path().join(".nvmrc"), "999.0.0").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("999.0.0"))
        .stdout(predicate::str::contains(".nvmrc"));
}

#[test]
fn detects_python_version_file() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".python-version"), "999.99.0").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("999.99.0"));
}

#[test]
fn detects_rust_toolchain_toml() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"999.0.0\"\n",
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("rust"))
        .stdout(predicate::str::contains("999.0.0"));
}

#[test]
fn detects_go_mod() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("go.mod"),
        "module example.com/foo\n\ngo 999.0\n",
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("go"))
        .stdout(predicate::str::contains("999.0"));
}

#[test]
fn detects_package_json_engines() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"name":"test","engines":{"node":">=999"}}"#,
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains(">=999"));
}

#[test]
fn detects_tool_versions() {
    let tmp = TempDir::new().unwrap();
    fs::write(
        tmp.path().join(".tool-versions"),
        "nodejs 999.0.0\npython 999.0.0\n",
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("python"));
}

#[test]
fn fix_flag_shows_suggestions() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".nvmrc"), "999.0.0").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .arg("--fix")
        .assert()
        .stdout(predicate::str::contains("Suggested fixes"))
        .stdout(predicate::str::contains("nvm install"));
}

#[test]
fn quiet_mode_exit_code_1_on_mismatch() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".nvmrc"), "999.0.0").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .arg("--quiet")
        .assert()
        .code(1)
        .stdout(predicate::str::is_empty());
}

#[test]
fn multiple_files_combined() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join(".nvmrc"), "999.0.0").unwrap();
    fs::write(tmp.path().join(".python-version"), "999.0.0").unwrap();
    fs::write(
        tmp.path().join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"999.0.0\"\n",
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("rust"));
}

#[test]
fn version_flag_works() {
    viscacha()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("viscacha"));
}

#[test]
fn nonexistent_dir_exits_gracefully() {
    viscacha()
        .arg("--dir")
        .arg("/tmp/viscacha-nonexistent-dir-12345")
        .assert()
        .success()
        .stdout(predicate::str::contains("No version constraint files found"));
}

#[test]
fn duplicate_tool_from_multiple_files() {
    let tmp = TempDir::new().unwrap();
    // Both .nvmrc and package.json define node
    fs::write(tmp.path().join(".nvmrc"), "999.0.0").unwrap();
    fs::write(
        tmp.path().join("package.json"),
        r#"{"name":"test","engines":{"node":">=999"}}"#,
    )
    .unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        // Should show both entries (one from each source)
        .stdout(predicate::str::contains(".nvmrc"))
        .stdout(predicate::str::contains("package.json"));
}

#[test]
fn malformed_toml_does_not_crash() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("rust-toolchain.toml"), "{{{invalid toml").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No version constraint files found"));
}

#[test]
fn malformed_json_does_not_crash() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("package.json"), "not json at all").unwrap();

    viscacha()
        .arg("--dir")
        .arg(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No version constraint files found"));
}

#[test]
fn help_flag_works() {
    viscacha()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("toolchain version checker"));
}
