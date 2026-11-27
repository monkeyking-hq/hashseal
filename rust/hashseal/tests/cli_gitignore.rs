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
fn cli_gitignore_flag_and_config_opt_in() {
    let dir = TestDir::new("hashseal-cli-gitignore-test");
    let root = dir.0.as_path();
    let root_str = root.to_str().unwrap();

    fs::write(root.join(".gitignore"), "AGENTS.local.md\nscratch/\n").unwrap();
    fs::write(root.join("AGENTS.md"), "# Tracked\n").unwrap();
    fs::write(root.join("AGENTS.local.md"), "# Local unsealed\n").unwrap();
    fs::create_dir_all(root.join("scratch")).unwrap();
    fs::write(root.join("scratch/CLAUDE.md"), "# Ignored dir\n").unwrap();

    let bin = env!("CARGO_BIN_EXE_hashseal");

    // 1. Seal instruct with --gitignore: only AGENTS.md is sealed.
    let st = Command::new(bin)
        .args(["seal", "--instruct", "--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());
    let local_content = fs::read_to_string(root.join("AGENTS.local.md")).unwrap();
    assert!(!local_content.contains("hashseal:"));

    // 2. Default check (without --gitignore) fails on unsealed AGENTS.local.md.
    let st = Command::new(bin)
        .args(["check", "--root", root_str])
        .status()
        .unwrap();
    assert!(!st.success());

    // 3. Check with --gitignore passes.
    let st = Command::new(bin)
        .args(["check", "--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());

    // 4. Config walk.respect_gitignore = true makes check pass without CLI flag.
    fs::write(
        root.join(".hashseal.json"),
        "{\"walk\": {\"respect_gitignore\": true}}\n",
    )
    .unwrap();
    let st = Command::new(bin)
        .args(["check", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());

    // Remove config override for explicit tree --gitignore test.
    fs::remove_file(root.join(".hashseal.json")).unwrap();

    // 5. Tree seal & verify with --gitignore excludes ignored files even if new ignored files appear.
    let st = Command::new(bin)
        .args(["seal", "--tree", "--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());
    fs::write(root.join("scratch/new.md"), "# Added later\n").unwrap();
    let st = Command::new(bin)
        .args(["verify", "--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());

    // 6. Unseal --instruct --gitignore strips AGENTS.md without touching ignored files.
    let st = Command::new(bin)
        .args(["unseal", "--instruct", "--gitignore", "--root", root_str])
        .status()
        .unwrap();
    assert!(st.success());
    let agents_content = fs::read_to_string(root.join("AGENTS.md")).unwrap();
    assert!(!agents_content.contains("hashseal:"));
}
