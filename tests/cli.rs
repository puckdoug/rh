use std::process::Command;

#[test]
fn help_lists_options_without_starting_chat() {
    for flag in ["--help", "-h"] {
        let output = Command::new(env!("CARGO_BIN_EXE_rh"))
            .arg(flag)
            .output()
            .expect("run help");

        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).expect("UTF-8 help output");
        assert!(stdout.contains("Usage:"));
        assert!(stdout.contains("--help"));
        assert!(stdout.contains("-v, --verbose"));
        assert!(!stdout.contains("got:"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn verbose_flags_enable_diagnostics_on_stderr() {
    for flag in ["--verbose", "-v"] {
        let output = Command::new(env!("CARGO_BIN_EXE_rh"))
            .arg(flag)
            .output()
            .expect("run verbose chat");

        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 diagnostics");
        assert!(stderr.contains("Starting chat"));
    }
}

#[test]
fn unknown_options_are_rejected() {
    let output = Command::new(env!("CARGO_BIN_EXE_rh"))
        .arg("--unknown")
        .output()
        .expect("run with unknown option");

    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 error output");
    assert!(stderr.contains("--unknown"));
}
