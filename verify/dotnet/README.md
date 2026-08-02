---
hashseal: "blake3:a1b69450239b8e9ed21f75279a782c0513a7d0d80a1c28a330e6b182b52b3118"
---
# hashseal-verify (.NET)

Zero-dependency HashSeal instruct document verifier (framework + pure C# BLAKE3).

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- .NET 8+ SDK (projects target `net10.0`; change TFM if needed)
- **No NuGet package references** for the verify library

## Test

```bash
cd verify/dotnet
dotnet run --project Hashseal.Verify.Test
dotnet run --project Hashseal.Verify.Test -- tree
```

Uses frozen vectors at `../vectors/instruct-v1.json` and `../vectors/tree-v1.json`.

## API

```csharp
using Hashseal.Verify;

var r = Check.CheckDocumentText(markdownText);
// r.Ok, r.Status ("valid"|"mismatch"|"missing_seal"|"invalid_format")

var h = Tree.HashTreeFileContent("a.txt", "hello\n");
var v = Tree.VerifyTreeInMemory(
    new Dictionary<string, string> { ["a.txt"] = "hello\n" },
    new[] { new LedgerEntryLike { Path = "a.txt", Digest = h.Digest } });
// v.Ok, v.Findings — every non-OK path named
```

Instruct: FULL canonical digest. Tree: LF text policy + in-memory verify (same as JS/core).

## Vendor

`Blake3.cs` is a pure-C# port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual).

```text
Copyright (c) 2026 MonkeyKing.dev
```
