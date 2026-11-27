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

/// Options controlling directory traversal.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WalkOptions {
    /// Respect `.gitignore` files and `.git/info/exclude` during traversal (opt-in; default `false`).
    pub respect_gitignore: bool,
}

/// Collect regular files under root. `match_file(rel_posix) -> bool`.
pub fn walk_files(
    root: &Path,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    walk_files_with(root, WalkOptions::default(), &mut match_file)
}

/// Collect regular files under root with [`WalkOptions`]. `match_file(rel_posix) -> bool`.
pub fn walk_files_with(
    root: &Path,
    options: WalkOptions,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<PathBuf>> {
    let entries = walk_entries_with(root, options, &mut match_file)?;
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
    walk_entries_with(root, WalkOptions::default(), &mut match_file)
}

/// Collect regular files and matched file/broken symlinks under root with [`WalkOptions`].
/// Does NOT recurse into directory symlinks.
pub fn walk_entries_with(
    root: &Path,
    options: WalkOptions,
    mut match_file: impl FnMut(&str) -> bool,
) -> std::io::Result<Vec<WalkEntry>> {
    let mut out = Vec::new();
    let mut ignore_stack = Vec::new();
    if options.respect_gitignore {
        let info_exclude = root.join(".git").join("info").join("exclude");
        if info_exclude.is_file() {
            if let Some(frame) = load_gitignore_frame(&info_exclude, String::new()) {
                ignore_stack.push(frame);
            }
        }
    }
    walk_rec(
        root,
        root,
        options,
        &mut ignore_stack,
        &mut match_file,
        &mut out,
    )?;
    out.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(out)
}

#[derive(Debug, Clone)]
struct GitIgnoreRule {
    negated: bool,
    dir_only: bool,
    anchored: bool,
    trailing_globstar: bool,
    pat_segs: Vec<String>,
}

#[derive(Debug, Clone)]
struct GitIgnoreFrame {
    base_dir: String,
    rules: Vec<GitIgnoreRule>,
}

fn load_gitignore_frame(path: &Path, base_dir: String) -> Option<GitIgnoreFrame> {
    let text = fs::read_to_string(path).ok()?;
    let rules = text.lines().filter_map(parse_gitignore_line).collect();
    Some(GitIgnoreFrame { base_dir, rules })
}

/// Strip unescaped trailing spaces while unescaping trailing `\ `.
fn trim_unescaped_trailing_spaces(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut end = chars.len();
    while end > 0 && chars[end - 1] == ' ' {
        // Count preceding backslashes to see if this space is escaped.
        let mut bs = 0usize;
        while end >= 2 + bs && chars[end - 2 - bs] == '\\' {
            bs += 1;
        }
        if bs % 2 == 1 {
            break;
        }
        end -= 1;
    }
    chars[..end].iter().collect()
}

fn unescape_simple(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(&next) = chars.peek() {
                if matches!(next, '#' | '!' | ' ' | '\\') {
                    out.push(next);
                    chars.next();
                    continue;
                }
            }
        }
        out.push(c);
    }
    out
}

fn parse_gitignore_line(raw: &str) -> Option<GitIgnoreRule> {
    let line = raw.strip_suffix('\r').unwrap_or(raw);
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let trimmed = trim_unescaped_trailing_spaces(line);
    if trimmed.is_empty() {
        return None;
    }

    let (negated, rest) = if let Some(after) = trimmed.strip_prefix('!') {
        (true, after)
    } else {
        (false, trimmed.as_str())
    };
    if rest.is_empty() {
        return None;
    }

    let (dir_only, rest) = if let Some(before) = rest.strip_suffix('/') {
        // Only treat as dir_only if trailing '/' is not escaped.
        if !before.ends_with('\\') {
            (true, before)
        } else {
            (false, rest)
        }
    } else {
        (false, rest)
    };
    if rest.is_empty() {
        return None;
    }

    let anchored = rest.contains('/');
    let rest = rest.strip_prefix('/').unwrap_or(rest);
    let pat_segs: Vec<String> = rest
        .split('/')
        .filter(|s| !s.is_empty())
        .map(unescape_simple)
        .collect();
    if pat_segs.is_empty() {
        return None;
    }

    let trailing_globstar =
        anchored && pat_segs.len() > 1 && pat_segs.last().map(|s| s.as_str()) == Some("**");

    Some(GitIgnoreRule {
        negated,
        dir_only,
        anchored,
        trailing_globstar,
        pat_segs,
    })
}

fn rule_matches(rule: &GitIgnoreRule, sub_rel: &str, is_dir: bool) -> bool {
    if rule.dir_only && !is_dir {
        return false;
    }
    if rule.anchored {
        let pat_refs: Vec<&str> = rule.pat_segs.iter().map(|s| s.as_str()).collect();
        let path_segs: Vec<&str> = sub_rel.split('/').filter(|s| !s.is_empty()).collect();
        if rule.trailing_globstar {
            // `foo/**` matches entries inside `foo/`, not `foo` itself, so `!foo/keep` works.
            let prefix = &pat_refs[..pat_refs.len() - 1];
            (1..path_segs.len()).any(|k| match_segments(prefix, &path_segs[..k]))
        } else {
            match_segments(&pat_refs, &path_segs)
        }
    } else {
        let base = sub_rel.rsplit('/').next().unwrap_or(sub_rel);
        match_segment(&rule.pat_segs[0], base)
    }
}

fn is_ignored(stack: &[GitIgnoreFrame], rel_path: &str, is_dir: bool) -> bool {
    let mut ignored = false;
    for frame in stack {
        let sub_rel = if frame.base_dir.is_empty() {
            rel_path
        } else {
            match rel_path.strip_prefix(&frame.base_dir) {
                Some(rest) => match rest.strip_prefix('/') {
                    Some(r) => r,
                    None => continue,
                },
                None => continue,
            }
        };
        if sub_rel.is_empty() {
            continue;
        }
        for rule in &frame.rules {
            if rule_matches(rule, sub_rel, is_dir) {
                ignored = !rule.negated;
            }
        }
    }
    ignored
}

fn rel_posix(root: &Path, path: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|r| r.to_string_lossy().replace('\\', "/"))
}

fn walk_rec(
    root: &Path,
    dir: &Path,
    options: WalkOptions,
    ignore_stack: &mut Vec<GitIgnoreFrame>,
    match_file: &mut impl FnMut(&str) -> bool,
    out: &mut Vec<WalkEntry>,
) -> std::io::Result<()> {
    let pushed_frame = if options.respect_gitignore {
        let gi = dir.join(".gitignore");
        if gi.is_file() {
            let base_dir = rel_posix(root, dir).unwrap_or_default();
            if let Some(frame) = load_gitignore_frame(&gi, base_dir) {
                ignore_stack.push(frame);
                true
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    };

    let rd = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => {
            if pushed_frame {
                ignore_stack.pop();
            }
            return Ok(());
        }
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
            if options.respect_gitignore {
                if let Some(rel_dir) = rel_posix(root, &path) {
                    if is_ignored(ignore_stack, &rel_dir, true) {
                        continue;
                    }
                }
            }
            walk_rec(root, &path, options, ignore_stack, match_file, out)?;
        } else if ft.is_symlink() {
            let meta = fs::metadata(&path);
            let is_dir_target = meta.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            if is_dir_target {
                // Never recurse into directory symlinks
                continue;
            }
            let broken = meta.is_err();
            if let Some(rel) = rel_posix(root, &path) {
                if options.respect_gitignore && is_ignored(ignore_stack, &rel, false) {
                    continue;
                }
                if match_file(&rel) {
                    out.push(WalkEntry {
                        path,
                        is_symlink: true,
                        broken_symlink: broken,
                    });
                }
            }
        } else if ft.is_file() {
            if let Some(rel) = rel_posix(root, &path) {
                if options.respect_gitignore && is_ignored(ignore_stack, &rel, false) {
                    continue;
                }
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
    if pushed_frame {
        ignore_stack.pop();
    }
    Ok(())
}

#[derive(Debug, Clone)]
enum SegTok {
    Char(char),
    Any,
    Star,
    Class {
        negated: bool,
        items: Vec<ClassItem>,
    },
}

#[derive(Debug, Clone, Copy)]
enum ClassItem {
    Single(char),
    Range(char, char),
}

fn parse_seg_tokens(pat: &str) -> Vec<SegTok> {
    let p: Vec<char> = pat.chars().collect();
    let mut out = Vec::with_capacity(p.len());
    let mut i = 0usize;
    while i < p.len() {
        match p[i] {
            '*' => {
                if !matches!(out.last(), Some(SegTok::Star)) {
                    out.push(SegTok::Star);
                }
                i += 1;
            }
            '?' => {
                out.push(SegTok::Any);
                i += 1;
            }
            '[' => {
                if let Some((tok, next_i)) = parse_char_class(&p, i) {
                    out.push(tok);
                    i = next_i;
                } else {
                    out.push(SegTok::Char('['));
                    i += 1;
                }
            }
            c => {
                out.push(SegTok::Char(c));
                i += 1;
            }
        }
    }
    out
}

fn parse_char_class(p: &[char], start: usize) -> Option<(SegTok, usize)> {
    let mut i = start + 1;
    if i >= p.len() {
        return None;
    }
    let negated = if p[i] == '!' || p[i] == '^' {
        i += 1;
        true
    } else {
        false
    };
    let mut items = Vec::new();
    let first_idx = i;
    while i < p.len() {
        if p[i] == ']' && i > first_idx {
            return Some((SegTok::Class { negated, items }, i + 1));
        }
        if i + 2 < p.len() && p[i + 1] == '-' && p[i + 2] != ']' {
            items.push(ClassItem::Range(p[i], p[i + 2]));
            i += 3;
        } else {
            items.push(ClassItem::Single(p[i]));
            i += 1;
        }
    }
    None
}

fn class_matches(negated: bool, items: &[ClassItem], ch: char) -> bool {
    let matched = items.iter().any(|item| match *item {
        ClassItem::Single(c) => c == ch,
        ClassItem::Range(lo, hi) => lo <= ch && ch <= hi,
    });
    if negated {
        !matched
    } else {
        matched
    }
}

/// Match a single path segment against a wildcard pattern supporting `*`, `?`, and `[...]`.
fn match_segment(pat: &str, seg: &str) -> bool {
    let toks = parse_seg_tokens(pat);
    let s: Vec<char> = seg.chars().collect();
    let m = toks.len();
    let n = s.len();
    let mut dp = vec![vec![false; n + 1]; m + 1];
    dp[0][0] = true;
    for i in 1..=m {
        if matches!(toks[i - 1], SegTok::Star) {
            dp[i][0] = dp[i - 1][0];
        }
    }
    for i in 1..=m {
        for j in 1..=n {
            match &toks[i - 1] {
                SegTok::Star => dp[i][j] = dp[i - 1][j] || dp[i][j - 1],
                SegTok::Any => dp[i][j] = dp[i - 1][j - 1],
                SegTok::Char(c) => dp[i][j] = dp[i - 1][j - 1] && *c == s[j - 1],
                SegTok::Class { negated, items } => {
                    dp[i][j] = dp[i - 1][j - 1] && class_matches(*negated, items, s[j - 1]);
                }
            }
        }
    }
    dp[m][n]
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

    fn rels_of(root: &Path, paths: Vec<PathBuf>) -> Vec<String> {
        paths
            .into_iter()
            .map(|p| {
                p.strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
            })
            .collect()
    }

    #[test]
    fn walk_default_includes_gitignored_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join(".gitignore"), "ignored.md\n").unwrap();
        fs::write(root.join("ignored.md"), "# ignored\n").unwrap();
        fs::write(root.join("kept.md"), "# kept\n").unwrap();

        let rels = rels_of(root, walk_files(root, |r| r.ends_with(".md")).unwrap());
        assert_eq!(rels, vec!["ignored.md", "kept.md"]);
    }

    #[test]
    fn walk_respects_gitignore_rules_and_negation() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(
            root.join(".gitignore"),
            "# comment\r\n\r\n\\#hash.md\r\n\\!bang.md\r\ntrail\\ \r\n*.log\r\n/root_only.md\r\nign_dir/\r\nfoo/**\r\n!foo/keep.md\r\nnote-[0-9].md\r\n",
        )
        .unwrap();
        fs::write(root.join("#hash.md"), "1").unwrap();
        fs::write(root.join("!bang.md"), "1").unwrap();
        fs::write(root.join("trail "), "1").unwrap();
        fs::write(root.join("a.log"), "1").unwrap();
        fs::write(root.join("root_only.md"), "1").unwrap();
        fs::write(root.join("ign_dir"), "file with same name as dir rule").unwrap();
        fs::write(root.join("note-3.md"), "1").unwrap();
        fs::write(root.join("note-a.md"), "1").unwrap();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/root_only.md"), "1").unwrap();
        fs::create_dir_all(root.join("foo")).unwrap();
        fs::write(root.join("foo/drop.md"), "1").unwrap();
        fs::write(root.join("foo/keep.md"), "1").unwrap();

        let opts = WalkOptions {
            respect_gitignore: true,
        };
        let rels = rels_of(root, walk_files_with(root, opts, |_| true).unwrap());
        assert!(rels.contains(&".gitignore".to_string()));
        assert!(rels.contains(&"ign_dir".to_string()));
        assert!(rels.contains(&"sub/root_only.md".to_string()));
        assert!(rels.contains(&"foo/keep.md".to_string()));
        assert!(rels.contains(&"note-a.md".to_string()));

        assert!(!rels.contains(&"#hash.md".to_string()));
        assert!(!rels.contains(&"!bang.md".to_string()));
        assert!(!rels.contains(&"trail ".to_string()));
        assert!(!rels.contains(&"a.log".to_string()));
        assert!(!rels.contains(&"root_only.md".to_string()));
        assert!(!rels.contains(&"foo/drop.md".to_string()));
        assert!(!rels.contains(&"note-3.md".to_string()));
    }

    #[test]
    fn walk_respects_nested_gitignore_and_git_info_exclude() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::create_dir_all(root.join(".git/info")).unwrap();
        fs::write(root.join(".git/info/exclude"), "*.tmp\n").unwrap();
        fs::write(root.join(".gitignore"), "*.md\n!AGENTS.md\n").unwrap();
        fs::write(root.join("AGENTS.md"), "# keep\n").unwrap();
        fs::write(root.join("notes.md"), "# drop\n").unwrap();
        fs::write(root.join("scratch.tmp"), "drop\n").unwrap();

        fs::create_dir_all(root.join("pkg/cache")).unwrap();
        fs::write(root.join("pkg/.gitignore"), "cache/\nlocal.txt\n").unwrap();
        fs::write(root.join("pkg/AGENTS.md"), "# keep\n").unwrap();
        fs::write(root.join("pkg/local.txt"), "drop\n").unwrap();
        fs::write(root.join("pkg/cache/AGENTS.md"), "# pruned\n").unwrap();
        fs::write(root.join("local.txt"), "keep at root\n").unwrap();

        let opts = WalkOptions {
            respect_gitignore: true,
        };
        let rels = rels_of(root, walk_files_with(root, opts, |_| true).unwrap());
        assert!(rels.contains(&"AGENTS.md".to_string()));
        assert!(rels.contains(&"pkg/AGENTS.md".to_string()));
        assert!(rels.contains(&"local.txt".to_string()));
        assert!(!rels.contains(&"notes.md".to_string()));
        assert!(!rels.contains(&"scratch.tmp".to_string()));
        assert!(!rels.contains(&"pkg/local.txt".to_string()));
        assert!(!rels.contains(&"pkg/cache/AGENTS.md".to_string()));
    }
}
