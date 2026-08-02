# Vendored dependencies (Python)

## blake3.py

Pure-Python port of the official **BLAKE3 reference implementation**
(`reference_impl/reference_impl.rs` from the BLAKE3-team/BLAKE3 repository).

- Spec: https://github.com/BLAKE3-team/BLAKE3-specs
- Reference: https://github.com/BLAKE3-team/BLAKE3/tree/master/reference_impl
- Original license: CC0 1.0 / Apache-2.0 (dual)
- Hash mode only (no keyed / KDF helpers needed by HashSeal verify)

**No pip packages.** Do not add `requirements.txt` runtime deps under `verify/python/`.

```text
Copyright (c) 2026 MonkeyKing.dev (this README only)
```
