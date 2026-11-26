//! Minimal recursive walk + deterministic segment glob matching (no walkdir/globset).

use std::fs;
use std::path::{Path, PathBuf};

/// Directory basenames never entered during a walk (performance + correctness).
///
/// Includes build/deps caches and **agent git-worktree containers**. Tools such as
/// Claude Code place full checkouts under `.claude/worktrees/<name>/`; other agents
/// use `.worktrees/` or a top-level `worktrees/`. Crawling those re-walks entire
/// nested trees (and often `node_modules` / `target` inside each) and is a common
/// multi-minute hang — skip by directory name before descending.
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
            // Agent / IDE parallel worktree containers (never crawl nested checkouts)
            | "worktrees"
            | ".worktrees"
    )
}

/// Entry yielded by [`walk_entries`], including symlink status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalkEntry {
    pub path: PathBuf,
    pub is_symlink: bool,
    pub broken_symlink: bool,
}

/// Collect regular files under root. `match_file(rel_posix) -> bool`.
pub fn walk_files(
    root: &Path,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    let entries = walk_entries(root, &mut match_file)?;
    Ok(entries
        .into_iter()
        .filter(|e| !e.is_symlink)
        .map(|e| e.path)
        .collect())
}

/// Collect regular files and matched file/broken symlinks under root.
/// Does NOT recurse into directory symlinks.
pub fn walk_entries(
    root: &Path,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<WalkEntry>> {
    let mut out = Vec::new();
    walk_rec(root, root, &mut match_file, &mut out)?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

fn walk_rec(
    root: &Path,
    dir: &Path,
    match_file: &mut impl FnMut(&str) -> bool,
    out: &mut Vec<WalkEntry>,
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
        } else if ft.is_symlink() {
            let meta = fs::metadata(&path);
            let is_dir_target = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            if is_dir_target {
                // Never recurse into directory symlinks
                continue;
            }
            let broken = meta.is_err();
            if let Ok(rel) = path.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if match_file(&rel) {
                    out.push(WalkEntry {
                        path,
                        is_symlink: true,
                        broken_symlink: broken,
                    });
                }
            }
        } else if ft.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let rel = rel.to_string_lossy().replace('\\', "/");
                if match_file(&rel) {
                    out.push(WalkEntry {
                        path,
                        is_symlink: false,
                        broken_symlink: false,
                    });
                }
            }
        }
    }
    Ok(())
}

/// Match a single path segment against a wildcard pattern supporting `*` and `?`.
fn match_segment(pat: &str, seg: &str) -> bool {
    let p: Vec<char> = pat.chars().collect();
    let s: Vec<char> = seg.chars().collect();
    let (mut pi, mut si) = (0usize, 0usize);
    let mut star_pi: Option<usize> = None;
    let mut star_si = 0usize;

    while si < s.len() {
        if pi < p.len() && (p[pi] == '?' || p[pi] == s[si]) {
            pi += 1;
            si += 1;
        } else if pi < p.len() && p[pi] == '*' {
            star_pi = Some(pi);
            star_si = si;
            pi += 1;
        } else if let Some(sp) = star_pi {
            pi = sp + 1;
            star_si += 1;
            si = star_si;
        } else {
            return false;
        }
    }
    while pi < p.len() && p[pi] == '*' {
        pi += 1;
    }
    pi == p.len()
}

/// Match pattern segments against path segments (`**` matches 0 or more segments).
fn match_segments(pat_segs: &[&str], path_segs: &[&str]) -> bool {
    let m = pat_segs.len();
    let n = path_segs.len();
    let mut dp = vec![vec![false; n + 1]; m + 1];
    dp[0][0] = true;
    for i in 1..=m {
        if pat_segs[i - 1] == "**" {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=m {
        let p = pat_segs[i - 1];
        for j in 1..=n {
            if p == "**" {
                dp[i][j] = dp[i - 1][j] || dp[i][j - 1];
            } else if dp[i - 1][j - 1] && match_segment(p, path_segs[j - 1]) {
                dp[i][j] = true;
            }
        }
    }
    dp[m][n]
}

/// Deterministic segment-based glob matcher supporting `**`, `*`, `?`, and basenames.
pub fn glob_match(pattern: &str, path: &str) -> bool {
    let pattern = pattern.trim_start_matches("./");
    let path = path.trim_start_matches("./");
    if pattern == "**/*" || pattern == "*" {
        return true;
    }
    if pattern.is_empty() {
        return path.is_empty();
    }

    // Bare exact filename without slashes or wildcards matches at root or any subdir
    if !pattern.contains('/') && !pattern.contains('*') && !pattern.contains('?') {
        return path == pattern || path.ends_with(&format!("/{pattern}"));
    }

    let pat_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    match_segments(&pat_segs, &path_segs)
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
    fn glob_segment_precision() {
        assert!(glob_match("docs/*.md", "docs/a.md"));
        assert!(!glob_match("docs/*.md", "docs/sub/a.md"));
        assert!(!glob_match("docs/*.md", "other/docs/a.md"));
        assert!(glob_match("a/**/b/*.txt", "a/b/c.txt"));
        assert!(glob_match("a/**/b/*.txt", "a/x/y/b/c.txt"));
        assert!(!glob_match("a/**/b/*.txt", "a/x/y/b/sub/c.txt"));
        assert!(glob_match("**/foo?.md", "dir/foo1.md"));
        assert!(!glob_match("**/foo?.md", "dir/foo12.md"));
    }

    #[test]
    fn glob_mdc_ext() {
        assert!(glob_match("**/*.mdc", ".cursor/rules/a.mdc"));
        assert!(!glob_match("**/*.mdc", "a.md"));
    }

    #[test]
    fn skips_agent_worktree_dir_names() {
        assert!(is_skipped_dir_name("worktrees"));
        assert!(is_skipped_dir_name(".worktrees"));
        assert!(is_skipped_dir_name(".git"));
        assert!(!is_skipped_dir_name(".claude"));
        assert!(!is_skipped_dir_name("skills"));
    }

    #[test]
    fn walk_does_not_descend_into_worktrees() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let skill = root.join(".claude").join("skills").join("x");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "# skill\n").unwrap();
        let wt = root
            .join(".claude")
            .join("worktrees")
            .join("agent-1")
            .join(".claude")
            .join("skills")
            .join("nested");
        fs::create_dir_all(&wt).unwrap();
        fs::write(wt.join("SKILL.md"), "# nested should be skipped\n").unwrap();
        let root_wt = root.join(".worktrees").join("other").join("pkg");
        fs::create_dir_all(&root_wt).unwrap();
        fs::write(root_wt.join("AGENTS.md"), "# skip\n").unwrap();

        let found = walk_files(root, |rel| rel.ends_with(".md")).unwrap();
        let rels: Vec<_> = found
            .iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect();
        assert_eq!(rels, vec![".claude/skills/x/SKILL.md".to_string()]);
    }
}
