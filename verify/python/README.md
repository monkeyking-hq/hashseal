---
hashseal: "blake3:9ac7978d565ef0b3ff7f9b669bcd616d93c3227afc7c80c2d7e458f9811604af"
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

result = check_document_text(markdown_text)
# {
#   "ok": bool,
#   "status": "valid" | "mismatch" | "missing_seal" | "invalid_format",
#   "algorithm": "blake3" | None,
#   "expected": "blake3:…" | None,
#   "actual": "blake3:…" | None,
#   "message": str | None,
# }
```

Digest check only (FULL canonical mode). GPG signature verification is not performed here.

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
python test/test_vectors.py
python test/test_tree_vectors.py
```

Uses frozen vectors at `../vectors/instruct-v1.json` and `../vectors/tree-v1.json`.

## Vendor

`vendor/blake3.py` is a pure-Python port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual). No `pip install` required.

```text
Copyright (c) 2026 MonkeyKing.dev
```
