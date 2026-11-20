//! Integration test: spawns the real binary (`CARGO_BIN_EXE_*`) so `main`
//! itself is exercised: the coverage gate counts it.

use std::process::Command;

#[test]
fn fold_check_missing_file_exits_nonzero_with_message() {
    let out = Command::new(env!("CARGO_BIN_EXE_origami-cli"))
        .args(["fold-check", "does/not/exist.fold"])
        .output();
    assert!(out.is_ok(), "binary runs: {out:?}");
    let Ok(out) = out else {
        return;
    };
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("no such file"), "stderr: {stderr}");
}

#[test]
fn help_lists_fold_check() {
    let out = Command::new(env!("CARGO_BIN_EXE_origami-cli"))
        .arg("--help")
        .output();
    assert!(out.is_ok(), "binary runs: {out:?}");
    let Ok(out) = out else {
        return;
    };
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("fold-check"), "stdout: {stdout}");
}
