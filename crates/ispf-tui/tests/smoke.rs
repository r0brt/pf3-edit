fn binary_path() -> String {
    env!("CARGO_BIN_EXE_pf3-edit").to_string()
}

#[test]
fn sample_fixture_exists_for_manual_smoke_runs() {
    let fixture = std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/sample.txt"
    ));
    assert!(fixture.exists());
}

#[test]
fn binary_starts_without_panicking() {
    let output = std::process::Command::new(binary_path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn binary_fails_for_missing_cli_file() {
    let output = std::process::Command::new(binary_path())
        .arg("/definitely/missing/ispf-fixture.txt")
        .output()
        .unwrap();

    assert!(
        !output.status.success(),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn binary_help_succeeds() {
    let output = std::process::Command::new(binary_path())
        .arg("--help")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("USAGE:"),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn binary_version_succeeds() {
    let output = std::process::Command::new(binary_path())
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "0.1.0");
}

#[test]
fn debug_keys_requires_an_interactive_terminal() {
    let output = std::process::Command::new(binary_path())
        .arg("--debug-keys")
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requires an interactive terminal"),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn binary_rejects_extra_arguments() {
    let output = std::process::Command::new(binary_path())
        .args(["one.txt", "two.txt"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("expected at most one file path"),
        "stdout:\n{}\n\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}
