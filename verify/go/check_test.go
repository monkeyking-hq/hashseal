package hashseal

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestBlake3Hello(t *testing.T) {
	sum := Sum256([]byte("hello"))
	hex := ""
	const digits = "0123456789abcdef"
	for _, b := range sum {
		hex += string([]byte{digits[b>>4], digits[b&0xf]})
	}
	want := "ea8f163db38682925e4491c5e58d4bb3506ef8c14eb78a86e908c5624a67200f"
	if hex != want {
		t.Fatalf("blake3(hello)=%s want %s", hex, want)
	}
}

func TestOfficialVectors(t *testing.T) {
	_, file, _, _ := runtime.Caller(0)
	vecPath := filepath.Join(filepath.Dir(file), "..", "vectors", "instruct-v1.json")
	data, err := os.ReadFile(vecPath)
	if err != nil {
		t.Fatal(err)
	}
	var doc struct {
		Cases []struct {
			ID     string `json:"id"`
			Kind   string `json:"kind"`
			Text   string `json:"text"`
			Expect struct {
				OK     bool   `json:"ok"`
				Status string `json:"status"`
				Digest string `json:"digest"`
			} `json:"expect"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(data, &doc); err != nil {
		t.Fatal(err)
	}
	for _, c := range doc.Cases {
		if c.Kind != "check" && c.Kind != "" {
			// some cases may be raw digests — still try check if text present
		}
		if c.Text == "" {
			continue
		}
		r := CheckDocumentText(c.Text)
		if r.OK != c.Expect.OK || r.Status != c.Expect.Status {
			t.Errorf("%s: ok=%v status=%s want ok=%v status=%s msg=%v",
				c.ID, r.OK, r.Status, c.Expect.OK, c.Expect.Status, r.Message)
			continue
		}
		if c.Expect.Digest != "" && r.Expected != nil && *r.Expected != c.Expect.Digest && r.OK {
			t.Errorf("%s: digest %s want %s", c.ID, *r.Expected, c.Expect.Digest)
		}
	}
}
