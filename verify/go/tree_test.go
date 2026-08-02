package hashseal

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestOfficialTreeVectors(t *testing.T) {
	_, file, _, _ := runtime.Caller(0)
	vecPath := filepath.Join(filepath.Dir(file), "..", "vectors", "tree-v1.json")
	data, err := os.ReadFile(vecPath)
	if err != nil {
		t.Fatal(err)
	}

	var doc struct {
		Spec              string   `json:"spec"`
		LineEndingsLfText *bool    `json:"line_endings_lf_text"`
		TextExtensions    []string `json:"text_extensions"`
		Cases             []struct {
			ID             string            `json:"id"`
			Kind           string            `json:"kind"`
			Path           string            `json:"path"`
			Content        string            `json:"content"`
			Files          map[string]string `json:"files"`
			LedgerEntries  []struct {
				Path   string `json:"path"`
				Digest string `json:"digest"`
				Size   int    `json:"size"`
			} `json:"ledger_entries"`
			Expect struct {
				Digest   string `json:"digest"`
				Size     int    `json:"size"`
				OK       bool   `json:"ok"`
				Checked  int    `json:"checked"`
				Findings []struct {
					Path     string  `json:"path"`
					Status   string  `json:"status"`
					Expected *string `json:"expected"`
					Actual   *string `json:"actual"`
				} `json:"findings"`
			} `json:"expect"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(data, &doc); err != nil {
		t.Fatal(err)
	}
	if doc.Spec != "tree-v1" {
		t.Fatalf("unexpected spec %s", doc.Spec)
	}

	lf := true
	if doc.LineEndingsLfText != nil {
		lf = *doc.LineEndingsLfText
	}
	opts := &TreeHashOpts{
		LineEndingsLfText: lf,
		TextExtensions:    TextExtensionsFromList(doc.TextExtensions),
	}

	for _, c := range doc.Cases {
		c := c
		t.Run(c.ID, func(t *testing.T) {
			switch c.Kind {
			case "raw_file_digest":
				r := HashTreeFileContent(c.Path, c.Content, opts)
				if r.Digest != c.Expect.Digest {
					t.Fatalf("digest: got %s want %s", r.Digest, c.Expect.Digest)
				}
				if r.Size != c.Expect.Size {
					t.Fatalf("size: got %d want %d", r.Size, c.Expect.Size)
				}
			case "verify_tree":
				ledger := make([]LedgerEntry, 0, len(c.LedgerEntries))
				for _, e := range c.LedgerEntries {
					ledger = append(ledger, LedgerEntry{Path: e.Path, Digest: e.Digest, Size: e.Size})
				}
				files := c.Files
				if files == nil {
					files = map[string]string{}
				}
				r := VerifyTreeInMemory(files, ledger, opts)
				if r.OK != c.Expect.OK {
					t.Fatalf("ok: got %v want %v", r.OK, c.Expect.OK)
				}
				if r.Checked != c.Expect.Checked {
					t.Fatalf("checked: got %d want %d", r.Checked, c.Expect.Checked)
				}
				want := c.Expect.Findings
				if len(r.Findings) != len(want) {
					t.Fatalf("findings.length: got %d want %d", len(r.Findings), len(want))
				}
				for i, w := range want {
					g := r.Findings[i]
					if g.Path != w.Path {
						t.Fatalf("finding[%d].path: got %s want %s", i, g.Path, w.Path)
					}
					if g.Status != w.Status {
						t.Fatalf("finding[%d].status: got %s want %s", i, g.Status, w.Status)
					}
					if !strPtrEq(g.Expected, w.Expected) {
						t.Fatalf("finding[%d].expected: got %v want %v", i, ptrStr(g.Expected), ptrStr(w.Expected))
					}
					if !strPtrEq(g.Actual, w.Actual) {
						t.Fatalf("finding[%d].actual: got %v want %v", i, ptrStr(g.Actual), ptrStr(w.Actual))
					}
				}
			default:
				t.Fatalf("unknown kind %s", c.Kind)
			}
		})
	}
}

func strPtrEq(a, b *string) bool {
	if a == nil && b == nil {
		return true
	}
	if a == nil || b == nil {
		return false
	}
	return *a == *b
}

func ptrStr(p *string) string {
	if p == nil {
		return "<nil>"
	}
	return *p
}
