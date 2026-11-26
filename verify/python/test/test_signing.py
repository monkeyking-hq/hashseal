#!/usr/bin/env python3
"""Run official signing-v2 vectors against check_document_text (stdlib only)."""

from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from check import check_document_text  # noqa: E402

VECTORS = ROOT.parent / "vectors" / "signing-v2.json"


def main() -> int:
    doc = json.loads(VECTORS.read_text(encoding="utf-8"))
    if doc.get("spec") != "signing-v2":
        print(f"unexpected spec {doc.get('spec')}", file=sys.stderr)
        return 1

    passed = 0
    failed = 0
    for c in doc["cases"]:
        try:
            r = check_document_text(
                c["text"],
                allowed_signers=c["allowed_signers"],
                path=c["path"],
                require_signature=c["require_signature"],
            )
            exp = c["expect"]
            assert r["ok"] == exp["ok"], f"ok={r['ok']} want {exp['ok']} ({r})"
            assert r["status"] == exp["status"], f"status={r['status']} want {exp['status']} ({r})"
            if "signer" in exp:
                assert r["signer"] == exp["signer"], f"signer={r['signer']} want {exp['signer']}"
            passed += 1
            print(f"ok  {c['id']}")
        except Exception as exc:  # noqa: BLE001
            failed += 1
            print(f"FAIL {c['id']}: {exc}", file=sys.stderr)

    print(f"\n{passed} passed, {failed} failed")
    return 0 if failed == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
