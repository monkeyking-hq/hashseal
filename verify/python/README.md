---
hashseal: "blake3:d3b7fc28ae4be21fe7b9132ca572ba6ea9f6f0242b845c8a95cc8bd245fa1234"
---
# hashseal-verify (Python)

Zero-dependency HashSeal instruct document verifier.

**Signed, Sealed, Delivered - I'm Yours.**

## Install

Copy this folder or use from the monorepo. **No pip dependencies.**

Python 3.9+.

```bash
python -c "from check import check_document_text; print(check_document_text(open('AGENTS.md',encoding='utf-8').read()))"
```

From the package directory (`verify/python/`), or:

```python
import sys
sys.path.insert(0, "verify/python")
from check import check_document_text
```

## API

```python
from check import check_document_text

result = check_document_text(
    markdown_text,
    # Optional:
    # allowed_signers=open(".hashseal/allowed_signers", encoding="utf-8").read(),
    # path="AGENTS.md",
    # require_signature=True,
    # unicode_policy="fail",  # "fail" | "warn" | "off"
)
# {
#   "ok": bool,
#   "status": "valid" | "mismatch" | "missing_seal" | "invalid_format" | "suspicious_unicode" | "missing_signature" | "bad_signature" | "untrusted_key" | "unverified_signature",
#   "algorithm": "blake3" | None,
#   "expected": "blake3:…" | None,
#   "actual": "blake3:…" | None,
#   "signer": str | None,
#   "message": str | None,
# }
```

Supports `instruct-v2` lossless raw-FM digests, Unicode smuggling checks, and pure-Python Ed25519 `SSHSIG` verification (`sshsig.py` + `vendor/ed25519.py`).

### Tree (in-memory)

```python
from tree import hash_tree_file_content, verify_tree_in_memory

h = hash_tree_file_content("src/a.txt", "hello\n")
# h["digest"], h["size"]

r = verify_tree_in_memory(
    {"src/a.txt": "hello\n"},
    [{"path": "src/a.txt", "digest": h["digest"], "size": h["size"]}],
)
# r["ok"], r["checked"], r["findings"]  — every non-OK path listed
```

## Tests

```bash
python3 test/test_vectors.py       # instruct-v2 (19 cases)
python3 test/test_tree_vectors.py  # tree-v1 (12 cases)
python3 test/test_signing.py       # signing-v2 (5 SSHSIG cases)
```

Uses frozen vectors at `../vectors/instruct-v2.json`, `../vectors/signing-v2.json`, and `../vectors/tree-v1.json`.

## Vendor

`vendor/blake3.py` is a pure-Python port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual). No `pip install` required.

```text
Copyright (c) 2026 MonkeyKing.dev
```
