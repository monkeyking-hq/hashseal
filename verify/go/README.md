---
hashseal: "blake3:4801ebf012c56db67cb9a44263a036860f39a5d7d152639074e4158a7b59cd75"
---
# hashseal-verify (Go)

Zero **external module** dependencies (stdlib + vendored BLAKE3 in-tree).

```bash
cd verify/go
go test ./...                    # instruct-v2 (19), signing-v2 (5), tree-v1 (12)
go run ./test/run_vectors.go     # standalone instruct-v2 runner
```

```go
import hashseal "github.com/hashseal/verify-go"

r := hashseal.CheckDocumentText(markdown)
// r.OK, r.Status, r.Expected, r.Actual

h := hashseal.HashTreeFileContent("src/a.txt", "hello\n", &hashseal.TreeHashOpts{LineEndingsLfText: true})
tv := hashseal.VerifyTreeInMemory(map[string]string{"src/a.txt": "hello\n"}, []hashseal.LedgerEntry{
    {Path: "src/a.txt", Digest: h.Digest, Size: h.Size},
}, &hashseal.TreeHashOpts{LineEndingsLfText: true})
// tv.OK, tv.Checked, tv.Findings
```

Tree vectors: `go test .` runs `TestOfficialTreeVectors` against `../vectors/tree-v1.json`.

```text
Copyright (c) 2026 MonkeyKing.dev
```
