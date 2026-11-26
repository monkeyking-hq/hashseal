---
hashseal: "blake3:69293d800fdf094050c149aeaf619f2f57c1f44a58c56b387c6bdb84c84419f4"
---
# hashseal-verify (Go)

Zero **external module** dependencies (stdlib + vendored BLAKE3 in-tree).

```bash
cd verify/go
go test .
go run ./test/   # instruct-v1 runner
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
