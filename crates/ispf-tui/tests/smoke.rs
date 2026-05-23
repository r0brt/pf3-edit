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
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ispf-tui"))
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
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_ispf-tui"))
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
