#!/usr/bin/env python3
"""
Run official instruct-v1 vectors against check_document_text.
Usage: python test/test_vectors.py
Zero pip deps (stdlib only).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from check import blake3_digest, check_document_text  # noqa: E402

VECTORS = ROOT.parent / "vectors" / "instruct-v1.json"


def main() -> int:
    doc = json.loads(VECTORS.read_text(encoding="utf-8"))
    if doc.get("spec") != "instruct-v1":
        print(f"unexpected spec {doc.get('spec')}", file=sys.stderr)
        return 1

    passed = 0
    failed = 0
    for c in doc["cases"]:
        try:
            if c["kind"] == "raw_digest":
                actual = blake3_digest(c["bytes_utf8"])["qualified"]
                assert_eq(actual, c["expect"]["digest"], f"{c['id']} digest")
            elif c["kind"] == "check":
                r = check_document_text(c["text"])
                assert_eq(r["ok"], c["expect"]["ok"], f"{c['id']} ok")
                assert_eq(r["status"], c["expect"]["status"], f"{c['id']} status")
                if c["expect"].get("digest") is not None:
                    assert_eq(r["actual"], c["expect"]["digest"], f"{c['id']} actual digest")
                    if r["ok"]:
                        assert_eq(
                            r["expected"], c["expect"]["digest"], f"{c['id']} expected digest"
                        )
                if c["expect"].get("expected") is not None:
                    assert_eq(r["expected"], c["expect"]["expected"], f"{c['id']} expected")
                if c["expect"].get("actual") is not None:
                    assert_eq(r["actual"], c["expect"]["actual"], f"{c['id']} actual")
            else:
                raise RuntimeError(f"unknown kind {c['kind']}")
            passed += 1
            print(f"ok  {c['id']}")
        except Exception as e:  # noqa: BLE001 — report each case
            failed += 1
            print(f"FAIL {c['id']}: {e}", file=sys.stderr)

    print(f"\n{passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


def assert_eq(a, b, label: str) -> None:
    if a != b:
        raise AssertionError(f"{label}: got {a!r} want {b!r}")


if __name__ == "__main__":
    raise SystemExit(main())
