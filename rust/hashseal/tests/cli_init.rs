use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct TestDir(PathBuf);

impl TestDir {
    fn new(name: &str) -> Self {
        let p = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&p);
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn cli_init_creates_config_hooks_and_lockfile() {
    let dir = TestDir::new("hashseal-cli-init-test");
    let root = dir.0.as_path();

    assert!(Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["init", "-q"])
        .status()
        .unwrap()
        .success());
    fs::write(root.join("AGENTS.md"), "# Agent Rules\n\nBe safe.\n").unwrap();

    let bin = env!("CARGO_BIN_EXE_hashseal");
    let st = Command::new(bin)
        .args([
            "init",
            "--root",
            root.to_str().unwrap(),
            "--git-hooks",
            "--claude-hooks",
            "--lock",
        ])
        .status()
        .unwrap();
    assert!(st.success());

    assert!(root.join(".hashseal.json").is_file());
    assert!(root.join(".hashseal/allowed_signers").is_file());
    assert!(root.join(".githooks/pre-commit").is_file());
    assert!(root.join(".claude/settings.json").is_file());
    assert!(root.join(".hashseal/hooks/claude-guard.sh").is_file());
    assert!(root.join(".hashseal.lock").is_file());

    let check_st = Command::new(bin)
        .args(["check", "--root", root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(check_st.success());
}
