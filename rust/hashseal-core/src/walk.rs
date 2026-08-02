//! Minimal recursive walk + glob-ish matching (no walkdir/globset).

use std::fs;
use std::path::{Path, PathBuf};

/// Default directory names always skipped when walking.
pub fn is_skipped_dir_name(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | "dist"
            | "build"
            | "vendor"
            | ".hashseal"
            | "hashseal-bundle"
    )
}

/// Collect files under root. `match_file(rel_posix) -> bool`.
pub fn walk_files(
    root: &Path,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    walk_rec(root, root, &mut match_file, &mut out)?;
    out.sort();
    Ok(out)
}

fn walk_rec(
    root: &Path,
    dir: &Path,
    match_file: &mut impl FnMut(&str) -> bool,
    out: &mut Vec<PathBuf>,
) -> std::io::Result<()> {
    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };
    for ent in rd.flatten() {
        let path = ent.path();
        let name = ent.file_name();
        let name = name.to_string_lossy();
        let ft = match ent.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_dir() {
            if is_skipped_dir_name(&name) {
                continue;
            }
            walk_rec(root, &path, match_file, out)?;
        } else if ft.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if match_file(&rel) {
                    out.push(path);
                }
            }
        }
    }
    Ok(())
}

/// Whether `path` is under directory `dir` (as a path segment prefix).
///
/// Examples: dir=`".claude"` matches `.claude/skills/x.md` and `pkg/.claude/CLAUDE.md`.
fn under_dir(path: &str, dir: &str) -> bool {
    path == dir || path.starts_with(&format!("{dir}/")) || path.contains(&format!("/{dir}/"))
}

/// Very small glob: supports `**`, `*`, directory trees, and suffix patterns like `**/*.md`.
///
/// Supported forms (common for instruct includes):
/// - `**/*` / `*`
/// - exact path or basename (`AGENTS.md`, `.github/copilot-instructions.md`)
/// - `**/name` exact file at any depth
/// - `**/*.ext` / `*.ext`
/// - `**/dir/**` any file under `dir`
/// - `**/dir/**/*` same
/// - `**/dir/**/*.ext` files with extension under `dir`
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let path = path.trim_start_matches("./");
    if pattern == "**/*" || pattern == "*" {
        return true;
    }

    // Exact (no wildcards)
    if !pattern.contains('*') && !pattern.contains('?') {
        return path == pattern || path.ends_with(&format!("/{pattern}"));
    }

    if let Some(suf) = pattern.strip_prefix("**/") {
        // **/dir/**/*.ext
        if let Some((dir, ext)) = suf.split_once("/**/*.") {
            if !dir.is_empty()
                && !dir.contains('*')
                && !dir.contains('?')
                && !ext.contains('*')
                && !ext.contains('?')
            {
                return under_dir(path, dir) && path.ends_with(&format!(".{ext}"));
            }
        }
        // **/dir/** or **/dir/**/*
        if let Some(dir) = suf
            .strip_suffix("/**/*")
            .or_else(|| suf.strip_suffix("/**"))
        {
            if !dir.is_empty() && !dir.contains('*') && !dir.contains('?') {
                return under_dir(path, dir);
            }
        }
        // **/*.ext
        if let Some(ext) = suf.strip_prefix("*.") {
            if !ext.contains('*') && !ext.contains('?') {
                return path.ends_with(&format!(".{ext}"));
            }
        }
        // **/exact-file (no wildcards in suffix)
        if !suf.contains('*') && !suf.contains('?') {
            return path == suf || path.ends_with(&format!("/{suf}"));
        }
    }

    if let Some(ext) = pattern.strip_prefix("*.") {
        if !ext.contains('*') && !ext.contains('?') {
            return path.ends_with(&format!(".{ext}"));
        }
    }

    // naive fallback: strip ** / * and require remaining fragment
    let stripped = pattern.replace("**/", "").replace('*', "");
    if !stripped.is_empty() {
        return path.contains(&stripped) || path.ends_with(&stripped);
    }
    path == pattern
}

pub fn any_glob(patterns: &[String], path: &str) -> bool {
    patterns.iter().any(|p| glob_match(p, path))
}

/// True if `path` matches any of the string patterns (borrowed).
pub fn any_glob_str(patterns: &[&str], path: &str) -> bool {
    patterns.iter().any(|p| glob_match(p, path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_md() {
        assert!(glob_match("**/*.md", "docs/AGENTS.md"));
        assert!(glob_match("**/*.md", "AGENTS.md"));
        assert!(!glob_match("**/*.md", "a.rs"));
    }

    #[test]
    fn glob_exact_nested() {
        assert!(glob_match("**/AGENTS.md", "AGENTS.md"));
        assert!(glob_match("**/AGENTS.md", "pkg/AGENTS.md"));
        assert!(!glob_match("**/AGENTS.md", "AGENTS.local.md"));
        assert!(glob_match(
            "**/.github/copilot-instructions.md",
            ".github/copilot-instructions.md"
        ));
    }

    #[test]
    fn glob_dir_tree() {
        assert!(glob_match("**/.claude/**", ".claude/skills/foo/SKILL.md"));
        assert!(glob_match(
            "**/.claude/**/*.md",
            ".claude/skills/foo/SKILL.md"
        ));
        assert!(!glob_match("**/.claude/**/*.md", ".claude/settings.json"));
        assert!(glob_match(
            "**/.cursor/rules/**/*.mdc",
            ".cursor/rules/rust.mdc"
        ));
        assert!(glob_match("**/.agents/**", "pkg/.agents/skills/x/SKILL.md"));
        assert!(!glob_match("**/.claude/**", "not-claude/x.md"));
    }

    #[test]
    fn glob_mdc_ext() {
        assert!(glob_match("**/*.mdc", ".cursor/rules/a.mdc"));
        assert!(!glob_match("**/*.mdc", "a.md"));
    }
}
