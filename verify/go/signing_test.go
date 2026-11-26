package hashseal

import (
	"encoding/json"
	"os"
	"path/filepath"
	"runtime"
	"testing"
)

func TestOfficialSigningVectors(t *testing.T) {
	_, file, _, _ := runtime.Caller(0)
	vecPath := filepath.Join(filepath.Dir(file), "..", "vectors", "signing-v2.json")
	data, err := os.ReadFile(vecPath)
	if err != nil {
		t.Fatal(err)
	}
	var doc struct {
		Spec  string `json:"spec"`
		Cases []struct {
			ID               string `json:"id"`
			Text             string `json:"text"`
			Path             string `json:"path"`
			AllowedSigners   string `json:"allowed_signers"`
			RequireSignature bool   `json:"require_signature"`
			Expect           struct {
				OK     bool   `json:"ok"`
				Status string `json:"status"`
				Signer string `json:"signer"`
			} `json:"expect"`
		} `json:"cases"`
	}
	if err := json.Unmarshal(data, &doc); err != nil {
		t.Fatal(err)
	}
	if doc.Spec != "signing-v2" {
		t.Fatalf("unexpected spec %s", doc.Spec)
	}
	for _, c := range doc.Cases {
		r := CheckDocumentTextWith(c.Text, CheckOptions{
			AllowedSigners:   c.AllowedSigners,
			Path:             c.Path,
			RequireSignature: c.RequireSignature,
		})
		if r.OK != c.Expect.OK || r.Status != c.Expect.Status {
			t.Errorf("%s: ok=%v status=%s want ok=%v status=%s msg=%v",
				c.ID, r.OK, r.Status, c.Expect.OK, c.Expect.Status, r.Message)
			continue
		}
		if c.Expect.Signer != "" && (r.Signer == nil || *r.Signer != c.Expect.Signer) {
			t.Errorf("%s: signer=%v want %s", c.ID, r.Signer, c.Expect.Signer)
		}
	}
}
