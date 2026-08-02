#!/usr/bin/env python3
"""
Run official tree-v1 vectors against in-memory tree verify.
Usage: python test/test_tree_vectors.py
Zero pip deps (stdlib only).
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from tree import hash_tree_file_content, verify_tree_in_memory  # noqa: E402

VECTORS = ROOT.parent / "vectors" / "tree-v1.json"


def main() -> int:
    doc = json.loads(VECTORS.read_text(encoding="utf-8"))
    if doc.get("spec") != "tree-v1":
        print(f"unexpected spec {doc.get('spec')}", file=sys.stderr)
        return 1

    lf_text = doc.get("line_endings_lf_text", True) is not False
    text_extensions = doc.get("text_extensions")

    passed = 0
    failed = 0
    for c in doc["cases"]:
        try:
            if c["kind"] == "raw_file_digest":
                r = hash_tree_file_content(
                    c["path"],
                    c["content"],
                    line_endings_lf_text=lf_text,
                    text_extensions=text_extensions,
                )
                assert_eq(r["digest"], c["expect"]["digest"], f"{c['id']} digest")
                assert_eq(r["size"], c["expect"]["size"], f"{c['id']} size")
            elif c["kind"] == "verify_tree":
                r = verify_tree_in_memory(
                    c.get("files") or {},
                    c.get("ledger_entries") or [],
                    line_endings_lf_text=lf_text,
                    text_extensions=text_extensions,
                )
                assert_eq(r["ok"], c["expect"]["ok"], f"{c['id']} ok")
                assert_eq(r["checked"], c["expect"]["checked"], f"{c['id']} checked")
                want = c["expect"].get("findings") or []
                assert_eq(len(r["findings"]), len(want), f"{c['id']} findings.length")
                for i, w in enumerate(want):
                    g = r["findings"][i]
                    assert_eq(g["path"], w["path"], f"{c['id']} finding[{i}].path")
                    assert_eq(g["status"], w["status"], f"{c['id']} finding[{i}].status")
                    assert_eq(
                        g["expected"], w.get("expected"), f"{c['id']} finding[{i}].expected"
                    )
                    assert_eq(g["actual"], w.get("actual"), f"{c['id']} finding[{i}].actual")
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
