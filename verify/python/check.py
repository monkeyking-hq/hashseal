"""
HashSeal instruct document check — FULL canonical mode (digest only).
Mirrors hashseal-core instruct algorithm. Zero pip dependencies.

Copyright (c) 2026 MonkeyKing.dev
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any, Callable, Dict, List, Optional, Tuple

# Local vendor (stdlib only)
_VENDOR = Path(__file__).resolve().parent / "vendor"
if str(_VENDOR) not in sys.path:
    sys.path.insert(0, str(_VENDOR))

from blake3 import blake3, blake3_hex  # type: ignore  # noqa: E402

SEAL_FIELD = "hashseal"
SIG_FIELD = "hashseal_sig"
KEY_ID_FIELD = "hashseal_key_id"
RESERVED = frozenset({SEAL_FIELD, SIG_FIELD, KEY_ID_FIELD})


def check_document_text(text: str, field: str = SEAL_FIELD) -> Dict[str, Any]:
    """Check a sealed instruct markdown document (text in memory).

    Returns dict with keys: ok, status, algorithm, expected, actual, message.
    status: valid | mismatch | missing_seal | invalid_format
    """
    doc = parse_document(text)
    if not doc["had_front_matter"]:
        actual = compute_digest(doc)
        return {
            "ok": False,
            "status": "missing_seal",
            "algorithm": "blake3",
            "expected": None,
            "actual": actual["qualified"],
            "message": "missing hashseal field",
        }
    seal_raw = extract_reserved_field(doc["fm_lines"], field)
    if seal_raw is None:
        actual = compute_digest(doc)
        return {
            "ok": False,
            "status": "missing_seal",
            "algorithm": "blake3",
            "expected": None,
            "actual": actual["qualified"],
            "message": "missing hashseal field",
        }
    expected = parse_digest(seal_raw)
    if expected is None:
        return {
            "ok": False,
            "status": "invalid_format",
            "algorithm": None,
            "expected": None,
            "actual": None,
            "message": f"invalid digest format: {seal_raw}",
        }
    if expected["algorithm"] != "blake3":
        return {
            "ok": False,
            "status": "invalid_format",
            "algorithm": expected["algorithm"],
            "expected": expected["qualified"],
            "actual": None,
            "message": f"unsupported algorithm: {expected['algorithm']}",
        }
    actual = compute_digest(doc)
    if actual["hex"] != expected["hex"] or actual["algorithm"] != expected["algorithm"]:
        return {
            "ok": False,
            "status": "mismatch",
            "algorithm": expected["algorithm"],
            "expected": expected["qualified"],
            "actual": actual["qualified"],
            "message": None,
        }
    return {
        "ok": True,
        "status": "valid",
        "algorithm": actual["algorithm"],
        "expected": expected["qualified"],
        "actual": actual["qualified"],
        "message": None,
    }


def blake3_digest(data: bytes | str) -> Dict[str, str]:
    if isinstance(data, str):
        data = data.encode("utf-8")
    hex_ = blake3_hex(data)
    return {"algorithm": "blake3", "hex": hex_, "qualified": f"blake3:{hex_}"}


def strip_bom(s: str) -> str:
    return s[1:] if s and ord(s[0]) == 0xFEFF else s


def normalize_lf(s: str) -> str:
    return s.replace("\r\n", "\n").replace("\r", "\n")


def parse_document(text: str) -> Dict[str, Any]:
    text = strip_bom(text)
    if text.startswith("---\n") or text.startswith("---\r\n"):
        after_open = text[5:] if text.startswith("---\r\n") else text[4:]
        search = after_open
        offset = 0
        while True:
            idx = search.find("\n---")
            if idx < 0:
                break
            after = search[idx + 1 :]
            rest = after[3:]
            closed = (
                len(rest) == 0
                or rest.startswith("\n")
                or rest.startswith("\r\n")
                or rest.startswith("\r")
            )
            if closed:
                fm_block = after_open[: offset + idx]
                body = after_open[idx + 1 + 3 :]
                if body.startswith("\r\n"):
                    body = body[2:]
                elif body.startswith("\n"):
                    body = body[1:]
                elif body.startswith("\r"):
                    body = body[1:]
                fm_lines = normalize_lf(fm_block).split("\n")
                return {
                    "fm_lines": fm_lines,
                    "had_front_matter": True,
                    "body_raw": body,
                }
            offset += idx + 1
            search = search[idx + 1 :]
    return {"fm_lines": [], "had_front_matter": False, "body_raw": text}


def is_reserved_key(key: str) -> bool:
    return key in RESERVED


def for_each_fm_entry(lines: List[str], f: Callable[[str, str], None]) -> None:
    i = 0
    n = len(lines)
    while i < n:
        line = lines[i]
        trimmed = line.strip()
        if trimmed == "" or trimmed.startswith("#"):
            i += 1
            continue
        if line.startswith(" ") or line.startswith("\t"):
            i += 1
            continue
        colon = trimmed.find(":")
        if colon >= 0:
            key = trimmed[:colon].strip()
            rest = trimmed[colon + 1 :].strip()
            if is_reserved_key(key):
                i += 1
                while i < n:
                    L = lines[i]
                    if L.startswith(" ") or L.startswith("\t"):
                        i += 1
                        continue
                    if L.strip() == "":
                        if i + 1 < n and (
                            lines[i + 1].startswith(" ") or lines[i + 1].startswith("\t")
                        ):
                            i += 1
                            continue
                        break
                    break
                continue
            if rest in ("|", ">", "|-", ">-"):
                val = ""
                i += 1
                while i < n and (lines[i].startswith(" ") or lines[i].startswith("\t")):
                    if val != "":
                        val += "\n"
                    val += lines[i].lstrip()
                    i += 1
                f(key, val)
                continue
            val = rest
            if val.startswith('"') and val.endswith('"') and len(val) >= 2:
                val = val[1:-1]
            f(key, val)
        i += 1


def fm_map(lines: List[str]) -> Dict[str, str]:
    m: Dict[str, str] = {}
    for_each_fm_entry(lines, lambda k, v: m.__setitem__(k, v))
    return m


def extract_reserved_field(lines: List[str], field: str) -> Optional[str]:
    i = 0
    n = len(lines)
    while i < n:
        trimmed = lines[i].strip()
        colon = trimmed.find(":")
        if colon >= 0:
            k = trimmed[:colon].strip()
            if k == field:
                rest = trimmed[colon + 1 :].strip()
                if rest in ("|", ">", "|-", ">-"):
                    val = ""
                    i += 1
                    while i < n:
                        L = lines[i]
                        empty = L.strip() == ""
                        indented = L.startswith(" ") or L.startswith("\t")
                        if indented or (
                            empty
                            and i + 1 < n
                            and (
                                lines[i + 1].startswith(" ") or lines[i + 1].startswith("\t")
                            )
                        ):
                            if empty:
                                val += "\n"
                                i += 1
                                continue
                            if val != "":
                                val += "\n"
                            val += L.lstrip()
                            i += 1
                            continue
                        break
                    return val
                if rest.startswith('"') and rest.endswith('"') and len(rest) >= 2:
                    rest = rest[1:-1]
                return rest
        i += 1
    return None


def canonical_fm_string(m: Dict[str, str]) -> str:
    keys = sorted(m.keys())
    parts: List[str] = []
    for k in keys:
        v = m[k]
        s = f"{k}: "
        if v == "" or ":" in v or "#" in v or " " in v:
            s += '"' + v.replace('"', '\\"') + '"'
        else:
            s += v
        s += "\n"
        parts.append(s)
    return "".join(parts)


def hash_payload(doc: Dict[str, Any]) -> bytes:
    body_lf = normalize_lf(doc["body_raw"])
    m = fm_map(doc["fm_lines"])
    if not m:
        return body_lf.encode("utf-8")
    payload = canonical_fm_string(m) + "\n" + body_lf
    return payload.encode("utf-8")


def compute_digest(doc: Dict[str, Any]) -> Dict[str, str]:
    return blake3_digest(hash_payload(doc))


def parse_digest(raw: str) -> Optional[Dict[str, str]]:
    s = str(raw).strip()
    if s.startswith('"') and s.endswith('"') and len(s) >= 2:
        s = s[1:-1]
    idx = s.find(":")
    if idx < 0:
        return None
    algorithm = s[:idx].lower()
    hex_ = s[idx + 1 :].strip().lower()
    if not hex_ or any(c not in "0123456789abcdef" for c in hex_):
        return None
    return {"algorithm": algorithm, "hex": hex_, "qualified": f"{algorithm}:{hex_}"}


# Public aliases matching JS SDK style
checkDocumentText = check_document_text
blake3Digest = blake3_digest
blake3Hex = blake3_hex

__all__ = [
    "SEAL_FIELD",
    "check_document_text",
    "checkDocumentText",
    "blake3_digest",
    "blake3Digest",
    "blake3_hex",
    "blake3Hex",
]
