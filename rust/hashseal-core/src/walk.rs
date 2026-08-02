//! Minimal recursive walk + glob-ish matching (no walkdir/globset).

use std::fs;
use std::path::{Path, PathBuf};

/// Default directory names always skipped when walking.
pub fn is_skipped_dir_name(name: &str) -> bool {
    matches!(
        name,
        ".git" | "target" | "node_modules" | "dist" | "build" | "vendor" | ".hashseal" | "hashseal-bundle"
    )
}

/// Collect files under root. `match_file(rel_posix) -> bool`.
pub fn walk_files(root: &Path, mut match_file: impl FnMut(&str) -> bool) -> std::io::Result<Vec<PathBuf>> {
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

/// Very small glob: supports `**`, `*`, and suffix patterns like `**/*.md`.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let path = path.trim_start_matches("./");
    if pattern == "**/*" || pattern == "*" {
        return true;
    }
    if let Some(suf) = pattern.strip_prefix("**/") {
        if !suf.contains('*') && !suf.contains('?') {
            return path == suf || path.ends_with(&format!("/{suf}")) || path.ends_with(suf);
        }
        if let Some(ext) = suf.strip_prefix("*.") {
            return path.ends_with(&format!(".{ext}"));
        }
    }
    if let Some(ext) = pattern.strip_prefix("*.") {
        return path.ends_with(&format!(".{ext}"));
    }
    // exact
    if !pattern.contains('*') {
        return path == pattern || path.ends_with(&format!("/{pattern}"));
    }
    // naive **/*.ext already handled; fallback: ends with pattern without stars
    let stripped = pattern.replace("**/", "").replace("*", "");
    if !stripped.is_empty() {
        return path.contains(&stripped) || path.ends_with(&stripped);
    }
    path == pattern
}

pub fn any_glob(patterns: &[String], path: &str) -> bool {
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
}
