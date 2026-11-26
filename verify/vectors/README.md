---
hashseal: "blake3:e57c884abb4b2e9c931efa9625e13ab7f55527334c61da08789a397add1ec94d"
---
# Official HashSeal test vectors

Generated from `hashseal-core` and required for every verify implementation (CLI, check, WASM, language SDKs).

## Files

| File | Spec | Notes |
|------|------|--------|
| [`instruct-v2.json`](./instruct-v2.json) | instruct-v2 | FULL canonical mode (lossless raw-FM), blake3, field `hashseal` (**19** cases) |
| [`tree-v1.json`](./tree-v1.json) | tree-v1 | Tree file hash + verify findings (**12** cases) |

## `instruct-v2.json`

- **canonical:** `full` — lossless raw non-reserved front-matter lines + LF-normalized body with `HASHSEAL-INSTRUCT-V2` domain separation.
- **Reserved FM keys** (excluded from hash): `hashseal` (or configured field), `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, `hashseal_key_id`.
- **Case kinds:**
  - `check` — run instruct check on `text`; assert `expect.ok`, `expect.status`, and digests.
  - `raw_digest` — hash `bytes_utf8` with blake3; assert `expect.digest`.

### Status values

| Status | Meaning |
|--------|---------|
| `valid` | Seal present and digests match |
| `mismatch` | Seal present but content digests differ |
| `missing_seal` | No `hashseal` field (or no front matter) |
| `invalid_format` | Seal present but digest string is not `alg:hex` (or unsupported by SDK) |
| `suspicious_unicode` | Hidden Unicode tag, BiDi override, or zero-width smuggling detected |

### Edge cases covered

Beyond simple body / FM / tamper / CRLF:

- Reserved `hashseal_sig`, `hashseal_sig_format`, `hashseal_signer`, `hashseal_scope`, and `hashseal_key_id` excluded from hash
- YAML lists and nested maps in front matter protected verbatim
- UTF-8 BOM stripped before parse
- Malformed digest → `invalid_format`
- BiDi / Unicode tag smuggling → `suspicious_unicode`

### Core regression

`hashseal-core` loads this file in unit tests (`instruct::tests::official_vectors_instruct_v2`). Digests must not change without a deliberate vector version bump.

### Language SDKs

```text
verify/js/     — zero npm deps; checkDocumentText(text)
verify/python/ — stdlib + vendor blake3
verify/java/   — pure Java
verify/go/     — pure Go
verify/ruby/   — pure Ruby
verify/dotnet/ — pure C#
```

Each SDK should fail a case by printing case `id` and expected vs actual digests.

All instruct runners must pass **19** cases from `instruct-v2.json`.

---

## `tree-v1.json`

**Status:** frozen. Digests produced by `hashseal-core` tree file-hash policy (`line_endings_lf_text: true`).

### Goals

1. Shared multi-lang vectors for **tree ledger** verify (same digests as `seal_tree` / `verify_tree`).
2. Failures name **every** non-OK path with status + digests (never exit-only).
3. `hashseal-check` stays blake3-only for instruct; tree vectors are exercised by full core + language SDKs (in-memory hash/verify parity).

### Case kinds

| Kind | Runner does |
|------|-------------|
| `raw_file_digest` | Hash `content` for `path` with text LF policy (see `text_extensions`); assert `expect.digest` and on-disk `size` |
| `verify_tree` | Materialize `files` (or use in-memory map) → compare to frozen `ledger_entries` with `include` / `exclude` → assert `expect.ok`, `checked`, and full `findings` |

### Finding status values

| Status | Meaning |
|--------|---------|
| `mismatch` | Path in ledger; content digest differs |
| `removed` | Path in ledger; missing on disk / in files map |
| `added` | Path on disk; not in ledger |

### LF / path policy (matches core)

- **Text extensions** (listed in the JSON): LF-normalize + strip UTF-8 BOM before blake3.
- **Non-text** (e.g. `.dat`): hash raw bytes (CRLF ≠ LF).
- **Size:** byte length of file content **before** normalization.
- **Paths:** forward slashes, relative to seal root, stable sort in ledger.
- Vectors use `include: ["**/*"]` and `exclude: []` so default component skips do not hide fixtures.

### Coverage

- Round-trip two files
- CRLF sealed / LF on disk still OK (text)
- Mismatch / removed / added
- Combined modify + add + remove (all three findings)
- Binary raw CRLF
- Empty text

### Core regression

`hashseal-core` loads this file in `tree::tests::official_vectors_tree_v1`. For `ok: true` cases, a fresh `seal_tree` must reproduce frozen ledger digests.

### Language SDKs

| SDK | Tree vectors |
|-----|----------------|
| `verify/js` | `tree.js` — `hashTreeFileContent` / `verifyTreeInMemory`; `node test/tree-vectors.test.js` (**12**/12) |
| `verify/python` | `tree.py`; `python test/test_tree_vectors.py` (**12**/12) |
| `verify/go` | `tree.go`; `go test .` → `TestOfficialTreeVectors` (**12**/12) |
| `verify/java` | `Tree.java`; `java -cp out RunTreeVectors` (**12**/12) |
| Ruby / .NET | Optional later; digests frozen in JSON |

### Non-goals for tree-v1

- GPG signing of ledger
- Remote/HTTP fetch of ledger
- Third-party integrity product formats

```text
Copyright (c) 2026 MonkeyKing.dev
```
