// In-memory tree verify — mirrors hashseal-core tree hash + verify policy.
// Zero external package deps. Used for multi-lang tree-v1 vectors.
//
// Copyright (c) 2026 MonkeyKing.dev
package hashseal

import (
	"sort"
	"strings"
)

// DefaultTextExtensions matches hashseal-core text LF policy extensions.
var DefaultTextExtensions = map[string]struct{}{
	"md": {}, "txt": {}, "toml": {}, "yml": {}, "yaml": {}, "json": {},
	"rs": {}, "java": {}, "go": {}, "py": {}, "js": {}, "ts": {}, "tsx": {},
	"jsx": {}, "css": {}, "html": {}, "xml": {}, "sh": {}, "ps1": {},
	"c": {}, "h": {}, "cpp": {}, "cs": {}, "rb": {}, "svg": {},
}

// TreeHashOpts controls tree file hashing (mirrors core config).
type TreeHashOpts struct {
	// LineEndingsLfText defaults to true when zero-value is used via HashTreeFileContent helpers.
	// Callers should set explicitly when building from vectors.
	LineEndingsLfText bool
	// TextExtensions: if nil, DefaultTextExtensions is used.
	TextExtensions map[string]struct{}
}

// TreeFileHash is the result of hashing one path under tree policy.
type TreeFileHash struct {
	Digest    string // qualified blake3:hex
	Qualified string
	Hex       string
	Size      int // on-disk UTF-8 byte length before normalize
}

// Finding is one non-OK path from tree verify.
type Finding struct {
	Path     string  `json:"path"`
	Status   string  `json:"status"` // mismatch | removed | added
	Expected *string `json:"expected"`
	Actual   *string `json:"actual"`
}

// TreeVerifyResult is the result of VerifyTreeInMemory.
type TreeVerifyResult struct {
	OK       bool
	Checked  int
	Findings []Finding
}

// LedgerEntry is a frozen ledger row used for verify.
type LedgerEntry struct {
	Path   string
	Digest string
	Size   int
}

func extOf(path string) string {
	i := strings.LastIndex(path, ".")
	if i < 0 {
		return ""
	}
	return strings.ToLower(path[i+1:])
}

func textExtSet(opts *TreeHashOpts) map[string]struct{} {
	if opts != nil && opts.TextExtensions != nil {
		return opts.TextExtensions
	}
	return DefaultTextExtensions
}

func lfTextEnabled(opts *TreeHashOpts) bool {
	if opts == nil {
		return true
	}
	return opts.LineEndingsLfText
}

// HashTreeFileContent hashes path+content with core tree policy.
func HashTreeFileContent(path, content string, opts *TreeHashOpts) TreeFileHash {
	textExts := textExtSet(opts)
	size := len([]byte(content))
	data := content
	if lfTextEnabled(opts) {
		if _, ok := textExts[extOf(path)]; ok {
			data = strings.TrimPrefix(data, "\ufeff")
			data = normalizeLf(data)
		}
	}
	d := Blake3DigestString(data)
	return TreeFileHash{
		Digest:    d.Qualified,
		Qualified: d.Qualified,
		Hex:       d.Hex,
		Size:      size,
	}
}

// VerifyTreeInMemory compares in-memory files to ledger entries (same findings as core verify_tree).
func VerifyTreeInMemory(files map[string]string, ledger []LedgerEntry, opts *TreeHashOpts) TreeVerifyResult {
	if files == nil {
		files = map[string]string{}
	}
	paths := make([]string, 0, len(files))
	for p := range files {
		paths = append(paths, p)
	}
	sort.Strings(paths)

	current := make(map[string]string, len(files))
	for _, p := range paths {
		h := HashTreeFileContent(p, files[p], opts)
		current[p] = h.Qualified
	}

	findings := make([]Finding, 0)
	expectedPaths := make(map[string]struct{}, len(ledger))
	for _, e := range ledger {
		expectedPaths[e.Path] = struct{}{}
		actual, ok := current[e.Path]
		if !ok {
			exp := e.Digest
			findings = append(findings, Finding{
				Path: e.Path, Status: "removed", Expected: &exp, Actual: nil,
			})
		} else if actual != e.Digest {
			exp := e.Digest
			act := actual
			findings = append(findings, Finding{
				Path: e.Path, Status: "mismatch", Expected: &exp, Actual: &act,
			})
		}
	}

	// Deterministic "added" order: sorted current paths
	curPaths := make([]string, 0, len(current))
	for p := range current {
		curPaths = append(curPaths, p)
	}
	sort.Strings(curPaths)
	for _, p := range curPaths {
		if _, ok := expectedPaths[p]; !ok {
			act := current[p]
			findings = append(findings, Finding{
				Path: p, Status: "added", Expected: nil, Actual: &act,
			})
		}
	}

	sort.Slice(findings, func(i, j int) bool {
		return findings[i].Path < findings[j].Path
	})
	return TreeVerifyResult{
		OK:       len(findings) == 0,
		Checked:  len(ledger),
		Findings: findings,
	}
}

// TextExtensionsFromList builds a set from a string slice (vector text_extensions).
func TextExtensionsFromList(exts []string) map[string]struct{} {
	m := make(map[string]struct{}, len(exts))
	for _, e := range exts {
		m[strings.ToLower(e)] = struct{}{}
	}
	return m
}
