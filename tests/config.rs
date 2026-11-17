use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

struct TestDir(PathBuf);

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "rh-config-test-{}-{}",
            std::process::id(),
            NEXT_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("create test directory");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent directory");
        fs::write(&path, contents).expect("write configuration");
        path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn command(root: &TestDir, cwd: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rh"));
    command
        .args(args)
        .current_dir(cwd)
        .env("HOME", root.path().join("home"))
        .env("XDG_CONFIG_HOME", root.path().join("user"))
        .env("XDG_CONFIG_DIRS", root.path().join("system"))
        .env("USERPROFILE", root.path().join("home"))
        .env("APPDATA", root.path().join("user"))
        .env("PROGRAMDATA", root.path().join("system"))
        .stdin(Stdio::null());
    command
}

fn run(root: &TestDir, cwd: &Path, args: &[&str]) -> Output {
    command(root, cwd, args).output().expect("run rh")
}

#[test]
fn loads_both_formats_from_project_root_and_json_takes_precedence() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project/.git")).unwrap();
    fs::create_dir_all(root.path().join("project/subdir")).unwrap();
    root.write("project/.rh/config.toml", "verbose = false\n");
    let json = root.write("project/.rh/config.json", r#"{"verbose": true}"#);

    let output = run(&root, &root.path().join("project/subdir"), &[]);
    assert!(output.status.success(), "{:?}", output);
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&json.display().to_string()), "{stderr}");
}

#[test]
fn project_settings_override_user_and_system_settings_and_cli_wins() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project")).unwrap();
    #[cfg(target_os = "macos")]
    root.write(
        "home/Library/Application Support/rh/config.toml",
        "verbose = true\n",
    );
    #[cfg(not(target_os = "macos"))]
    root.write("user/rh/config.toml", "verbose = true\n");
    root.write("system/rh/config.json", r#"{"verbose": true}"#);
    root.write("project/.rh/config.toml", "verbose = false\n");

    let quiet = run(&root, &root.path().join("project"), &[]);
    assert!(quiet.status.success(), "{:?}", quiet);
    assert!(quiet.stderr.is_empty(), "{:?}", quiet);

    let verbose = run(&root, &root.path().join("project"), &["-v"]);
    assert!(verbose.status.success(), "{:?}", verbose);
    assert!(
        String::from_utf8(verbose.stderr)
            .unwrap()
            .contains("Starting chat")
    );
}

#[test]
fn loads_user_configuration_without_a_project_config() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project")).unwrap();
    #[cfg(target_os = "macos")]
    let path = root.write(
        "home/Library/Application Support/rh/config.toml",
        "verbose = true\n",
    );
    #[cfg(not(target_os = "macos"))]
    let path = root.write("user/rh/config.toml", "verbose = true\n");

    let output = run(&root, &root.path().join("project"), &[]);
    assert!(output.status.success(), "{:?}", output);
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[test]
fn rejects_invalid_configuration_with_filename() {
    let root = TestDir::new();
    let path = root.write("project/.rh/config.json", r#"{"verbose": "yes"}"#);
    let output = run(&root, &root.path().join("project"), &[]);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    assert!(stderr.contains("boolean"), "{stderr}");
}

#[test]
fn rejects_unknown_settings() {
    let root = TestDir::new();
    root.write("project/.rh/config.toml", "verbsoe = true\n");
    let output = run(&root, &root.path().join("project"), &[]);
    assert!(!output.status.success());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("verbsoe")
    );
}

#[test]
fn loads_project_agents_md_from_git_root_or_worktree() {
    for worktree in [false, true] {
        let root = TestDir::new();
        if worktree {
            root.write("project/.git", "gitdir: /unused/worktree\n");
        } else {
            fs::create_dir_all(root.path().join("project/.git")).unwrap();
        }
        fs::create_dir_all(root.path().join("project/subdir")).unwrap();
        let text = "# Instructions\nKeep café names unchanged.\n";
        root.write("project/AGENTS.md", text);
        root.write("project/subdir/AGENTS.md", "not the root instructions");

        let output = run(&root, &root.path().join("project/subdir"), &["-v"]);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        let expected = format!(
            "{} ({} bytes)",
            Path::new("project").join("AGENTS.md").display(),
            text.len()
        );
        assert!(stderr.contains("Loaded project instructions:"), "{stderr}");
        assert!(stderr.contains(&expected), "{stderr}");
        assert!(!stderr.contains(text), "{stderr}");
        assert!(
            !stderr.contains(&Path::new("subdir").join("AGENTS.md").display().to_string()),
            "{stderr}"
        );
    }
}

#[test]
fn loads_agents_md_from_cwd_outside_git_without_changing_normal_output() {
    let root = TestDir::new();
    root.write("project/AGENTS.md", "# Project instructions\n");
    let cwd = root.path().join("project");
    let quiet = run(&root, &cwd, &[]);
    assert!(quiet.status.success(), "{quiet:?}");
    assert!(quiet.stdout.is_empty(), "{quiet:?}");
    assert!(quiet.stderr.is_empty(), "{quiet:?}");
    let verbose = run(&root, &cwd, &["-v"]);
    assert!(verbose.status.success(), "{verbose:?}");
    let stderr = String::from_utf8(verbose.stderr).unwrap();
    assert!(stderr.contains("Loaded project instructions:"), "{stderr}");
    assert!(
        stderr.contains(&Path::new("project").join("AGENTS.md").display().to_string()),
        "{stderr}"
    );
}

#[test]
fn empty_agents_md_is_loaded() {
    let root = TestDir::new();
    root.write("project/AGENTS.md", "");
    let output = run(&root, &root.path().join("project"), &["-v"]);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("AGENTS.md (0 bytes)"), "{stderr}");
}

#[test]
fn reports_agents_md_read_errors_with_filename() {
    let root = TestDir::new();
    let path = root.path().join("project/AGENTS.md");
    fs::create_dir_all(&path).unwrap();
    let output = run(&root, &root.path().join("project"), &[]);
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[test]
fn rejects_non_utf8_agents_md_with_filename() {
    let root = TestDir::new();
    let path = root.write("project/AGENTS.md", "");
    fs::write(&path, [0xff]).unwrap();
    let output = run(&root, &root.path().join("project"), &[]);
    assert!(!output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[test]
fn only_loads_agents_md_at_nearest_project_root() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project/.git")).unwrap();
    fs::create_dir_all(root.path().join("project/nested/.git")).unwrap();
    root.write("project/AGENTS.md", "outer instructions");
    root.write(
        "project/nested/.rh/AGENTS.md",
        "config directory instructions",
    );
    root.write("user/rh/AGENTS.md", "user instructions");

    let output = run(&root, &root.path().join("project/nested"), &["-v"]);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(!stderr.contains("Loaded project instructions"), "{stderr}");
}

#[test]
fn missing_configuration_uses_quiet_defaults() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project")).unwrap();

    let output = run(&root, &root.path().join("project"), &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn loads_system_configuration() {
    let root = TestDir::new();
    let path = root.write("system/rh/config.json", r#"{"verbose": true}"#);

    let output = run(&root, root.path(), &[]);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[test]
fn user_configuration_overrides_system_configuration() {
    let root = TestDir::new();
    root.write(
        "system/rh/config.toml",
        "verbose = true
",
    );
    root.write("user/rh/config.json", r#"{"verbose": false}"#);

    let output = run(&root, root.path(), &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn empty_project_configuration_preserves_user_settings() {
    let root = TestDir::new();
    root.write(
        "user/rh/config.toml",
        "verbose = true
",
    );
    let path = root.write("project/.rh/config.json", "{}");

    let output = run(&root, &root.path().join("project"), &[]);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[test]
fn uses_current_directory_outside_git() {
    let root = TestDir::new();
    root.write(
        "project/.rh/config.toml",
        "verbose = true
",
    );
    fs::create_dir_all(root.path().join("project/subdir")).unwrap();

    let project = run(&root, &root.path().join("project"), &[]);
    assert!(project.status.success(), "{project:?}");
    assert!(
        String::from_utf8(project.stderr)
            .unwrap()
            .contains("Starting chat")
    );

    let subdir = run(&root, &root.path().join("project/subdir"), &[]);
    assert!(subdir.status.success(), "{subdir:?}");
    assert!(subdir.stderr.is_empty(), "{subdir:?}");
}

#[test]
fn uses_nearest_git_root_and_recognizes_worktree_files() {
    let root = TestDir::new();
    fs::create_dir_all(root.path().join("project/.git")).unwrap();
    root.write(
        "project/.rh/config.toml",
        "verbose = false
",
    );
    root.write(
        "project/worktree/.git",
        "gitdir: /unused/worktree
",
    );
    let path = root.write(
        "project/worktree/.rh/config.toml",
        "verbose = true
",
    );
    fs::create_dir_all(root.path().join("project/worktree/subdir")).unwrap();

    let output = run(&root, &root.path().join("project/worktree/subdir"), &[]);
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(not(windows))]
#[test]
fn falls_back_to_home_config_for_missing_empty_or_relative_xdg_home() {
    let root = TestDir::new();
    let path = root.write(
        "home/.config/rh/config.toml",
        "verbose = true
",
    );
    root.write("relative/rh/config.toml", "invalid syntax");

    for value in [None, Some(""), Some("relative")] {
        let mut cmd = command(&root, root.path(), &[]);
        match value {
            Some(value) => {
                cmd.env("XDG_CONFIG_HOME", value);
            }
            None => {
                cmd.env_remove("XDG_CONFIG_HOME");
            }
        }
        let output = cmd.output().unwrap();
        assert!(output.status.success(), "{value:?}: {output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("Starting chat"), "{value:?}: {stderr}");
        assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    }
}

#[cfg(not(windows))]
#[test]
fn first_xdg_system_directory_has_priority_and_relative_entries_are_ignored() {
    let root = TestDir::new();
    let high = root.write(
        "high/rh/config.toml",
        "verbose = true
",
    );
    let low = root.write(
        "low/rh/config.toml",
        "verbose = false
",
    );
    root.write("relative/rh/config.toml", "invalid syntax");
    let directories = std::env::join_paths([
        root.path().join("high"),
        PathBuf::from("relative"),
        root.path().join("low"),
    ])
    .unwrap();

    let output = command(&root, root.path(), &[])
        .env("XDG_CONFIG_DIRS", directories)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    let high_position = stderr.find(&high.display().to_string()).unwrap();
    let low_position = stderr.find(&low.display().to_string()).unwrap();
    assert!(low_position < high_position, "{stderr}");
}

#[cfg(target_os = "macos")]
#[test]
fn macos_library_configuration_overrides_xdg_user_configuration() {
    let root = TestDir::new();
    root.write(
        "user/rh/config.toml",
        "verbose = true
",
    );
    root.write(
        "home/Library/Application Support/rh/config.toml",
        "verbose = false
",
    );

    let output = run(&root, root.path(), &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[cfg(windows)]
#[test]
fn windows_loads_profile_configuration_without_appdata() {
    let root = TestDir::new();
    let path = root.write(
        "home/.config/rh/config.toml",
        "verbose = true
",
    );

    let output = command(&root, root.path(), &[])
        .env_remove("APPDATA")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("Starting chat"), "{stderr}");
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}

#[cfg(windows)]
#[test]
fn windows_appdata_overrides_profile_configuration() {
    let root = TestDir::new();
    root.write(
        "home/.config/rh/config.toml",
        "verbose = true
",
    );
    root.write(
        "user/rh/config.toml",
        "verbose = false
",
    );

    let output = run(&root, root.path(), &[]);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}

#[test]
fn rejects_malformed_syntax_in_either_format() {
    for name in ["config.toml", "config.json"] {
        let root = TestDir::new();
        let path = root.write(&format!("project/.rh/{name}"), "{");

        let output = run(&root, &root.path().join("project"), &[]);
        assert!(!output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(&path.display().to_string()), "{stderr}");
    }
}

#[test]
fn reports_configuration_read_errors_instead_of_ignoring_them() {
    let root = TestDir::new();
    let path = root.path().join("project/.rh/config.toml");
    fs::create_dir_all(&path).unwrap();

    let output = run(&root, &root.path().join("project"), &[]);
    assert!(!output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains(&path.display().to_string()), "{stderr}");
}
