---
hashseal: "blake3:211fd0fc4d1cd661f2930299770fd0f684feb8a7b52b5bbe96dc39c2812f7f61"
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
