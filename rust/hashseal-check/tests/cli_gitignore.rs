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
fn hashseal_check_respects_gitignore_only_when_enabled() {
    let dir = TestDir::new("hashseal-check-gitignore-test");
    let root = dir.0.as_path();
    let root_str = root.to_str().unwrap();

    fs::write(root.join(".gitignore"), "AGENTS.local.md\n").unwrap();
    // Minimal sealed AGENTS.md (pre-sealed with instruct-v2? Wait: use hashseal_core to seal!)
    let (sealed, _) = hashseal_core::seal_instruct_bytes(
        "# Tracked\n",
        &hashseal_core::InstructOptions::default(),
    )
    .unwrap();
    fs::write(root.join("AGENTS.md"), sealed).unwrap();
    fs::write(root.join("AGENTS.local.md"), "# Unsealed local\n").unwrap();

    let bin = env!("CARGO_BIN_EXE_hashseal-check");

    // Without --gitignore: fails on unsealed AGENTS.local.md
    let st = Command::new(bin)
        .args(["--root", root_str])
        .status()
        .unwrap();
    assert!(!st.success());

    // With --gitignore: passes
    let st = Command::new(bin)
        .args(["--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());
}
