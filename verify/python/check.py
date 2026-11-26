"""
HashSeal instruct document check — instruct-v2 (lossless raw-FM + Unicode + SSHSIG).
Mirrors hashseal-core instruct.rs & unicode.rs. Zero pip dependencies.

Copyright (c) 2026 MonkeyKing.dev
"""

from __future__ import annotations

import sys
from pathlib import Path
from typing import Any, Dict, List, Optional

_HERE = Path(__file__).resolve().parent
_VENDOR = _HERE / "vendor"
for p in (str(_HERE), str(_VENDOR)):
    if p not in sys.path:
        sys.path.insert(0, p)

from blake3 import blake3, blake3_hex  # type: ignore  # noqa: E402
from sshsig import verify_sshsig  # type: ignore  # noqa: E402

SEAL_FIELD = "hashseal"
SIG_FIELD = "hashseal_sig"
SIG_FORMAT_FIELD = "hashseal_sig_format"
SIGNER_FIELD = "hashseal_signer"
SCOPE_FIELD = "hashseal_scope"
KEY_ID_FIELD = "hashseal_key_id"
INSTRUCT_V2_DOMAIN = "HASHSEAL-INSTRUCT-V2"


def is_reserved_key(key: str, seal_field: str) -> bool:
    return key == seal_field or key in (
        SEAL_FIELD,
        SIG_FIELD,
        SIG_FORMAT_FIELD,
        SIGNER_FIELD,
        SCOPE_FIELD,
        KEY_ID_FIELD,
    )


def scan_suspicious_unicode(text: str) -> List[Dict[str, Any]]:
    findings: List[Dict[str, Any]] = []
    line = 1
    col = 1
    for idx, ch in enumerate(text):
        cp = ord(ch)
        kind: Optional[str] = None
        if 0xE0000 <= cp <= 0xE007F:
            kind = "unicode_tag"
        elif (
            (0x202A <= cp <= 0x202E)
            or (0x2066 <= cp <= 0x2069)
            or cp in (0x061C, 0x200E, 0x200F)
        ):
            kind = "bidi_control"
        elif cp in (0x200B, 0x2060) or (0x2061 <= cp <= 0x2064):
            kind = "zero_width"
        elif cp == 0xFEFF and idx > 0:
            kind = "embedded_bom"
        if kind is not None:
            findings.append({"line": line, "col": col, "codepoint": cp, "kind": kind})
        if ch == "\n":
            line += 1
            col = 1
        else:
            col += 1
    return findings


def _format_unicode_findings(findings: List[Dict[str, Any]]) -> str:
    if not findings:
        return ""
    f = findings[0]
    head = f"U+{f['codepoint']:04X} ({f['kind']}) at {f['line']}:{f['col']}"
    if len(findings) > 1:
        return f"{head} (+{len(findings) - 1} more)"
    return head


def check_document_text(
    text: str,
    field: str = SEAL_FIELD,
    *,
    unicode_policy: str = "fail",
    require_signature: bool = False,
    digest_only: bool = False,
    allowed_signers: Optional[str] = None,
    path: Optional[str] = None,
) -> Dict[str, Any]:
    """Check a sealed instruct markdown document (text in memory)."""
    if unicode_policy == "fail":
        u = scan_suspicious_unicode(text)
        if u:
            return {
                "ok": False,
                "status": "suspicious_unicode",
                "algorithm": None,
                "expected": None,
                "actual": None,
                "signer": None,
                "message": _format_unicode_findings(u),
            }

    doc = parse_document(text)
    if not doc["had_front_matter"]:
        actual = compute_digest(doc, field)
        return {
            "ok": False,
            "status": "missing_seal",
            "algorithm": "blake3",
            "expected": None,
            "actual": actual["qualified"],
            "signer": None,
            "message": "missing hashseal field",
        }
    seal_raw = extract_reserved_field(doc["fm_lines"], field)
    if seal_raw is None:
        actual = compute_digest(doc, field)
        return {
            "ok": False,
            "status": "missing_seal",
            "algorithm": "blake3",
            "expected": None,
            "actual": actual["qualified"],
            "signer": None,
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
            "signer": None,
            "message": f"invalid digest format: {seal_raw}",
        }
    if expected["algorithm"] != "blake3":
        return {
            "ok": False,
            "status": "invalid_format",
            "algorithm": expected["algorithm"],
            "expected": expected["qualified"],
            "actual": None,
            "signer": None,
            "message": f"unsupported algorithm: {expected['algorithm']}",
        }
    actual = compute_digest(doc, field)
    if actual["hex"] != expected["hex"] or actual["algorithm"] != expected["algorithm"]:
        return {
            "ok": False,
            "status": "mismatch",
            "algorithm": expected["algorithm"],
            "expected": expected["qualified"],
            "actual": actual["qualified"],
            "signer": None,
            "message": None,
        }

    sig = extract_reserved_field(doc["fm_lines"], SIG_FIELD)
    if require_signature and sig is None:
        return {
            "ok": False,
            "status": "missing_signature",
            "algorithm": actual["algorithm"],
            "expected": expected["qualified"],
            "actual": None,
            "signer": None,
            "message": "missing hashseal_sig",
        }

    if sig is not None and not digest_only and (allowed_signers is not None or require_signature):
        declared_scope = extract_reserved_field(doc["fm_lines"], SCOPE_FIELD)
        if (
            declared_scope is not None
            and path is not None
            and declared_scope != "*"
            and declared_scope != path
        ):
            return {
                "ok": False,
                "status": "bad_signature",
                "algorithm": actual["algorithm"],
                "expected": expected["qualified"],
                "actual": None,
                "signer": None,
                "message": f"signature scope mismatch: signed for '{declared_scope}', checked at '{path}'",
            }
        verify_scope = declared_scope if declared_scope is not None else (path or "*")
        if allowed_signers is None:
            return {
                "ok": False,
                "status": "unverified_signature",
                "algorithm": actual["algorithm"],
                "expected": expected["qualified"],
                "actual": None,
                "signer": None,
                "message": "missing allowed_signers for signature verification",
            }
        sig_payload = f"HASHSEAL-SIG-V2\ndigest={actual['qualified']}\nscope={verify_scope}\n"
        res = verify_sshsig(sig_payload, sig, allowed_signers)
        if res["status"] == "good":
            return {
                "ok": True,
                "status": "valid",
                "algorithm": actual["algorithm"],
                "expected": expected["qualified"],
                "actual": actual["qualified"],
                "signer": res.get("signer"),
                "message": None,
            }
        return {
            "ok": False,
            "status": "untrusted_key" if res["status"] == "untrusted_key" else "bad_signature",
            "algorithm": actual["algorithm"],
            "expected": expected["qualified"],
            "actual": None,
            "signer": None,
            "message": res.get("message"),
        }

    return {
        "ok": True,
        "status": "valid",
        "algorithm": actual["algorithm"],
        "expected": expected["qualified"],
        "actual": actual["qualified"],
        "signer": None,
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
                body = after_open[offset + idx + 1 + 3 :]
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


def extract_reserved_field(lines: List[str], field: str) -> Optional[str]:
    i = 0
    n = len(lines)
    while i < n:
        if not lines[i].startswith((" ", "\t")):
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
                                    lines[i + 1].startswith(" ")
                                    or lines[i + 1].startswith("\t")
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


def strip_reserved_fm_lines(lines: List[str], seal_field: str) -> List[str]:
    out: List[str] = []
    i = 0
    n = len(lines)
    while i < n:
        trimmed = lines[i].strip()
        colon = trimmed.find(":")
        if (
            colon >= 0
            and not lines[i].startswith((" ", "\t"))
            and is_reserved_key(trimmed[:colon].strip(), seal_field)
        ):
            i += 1
            while i < n:
                L = lines[i]
                if L.startswith((" ", "\t")):
                    i += 1
                    continue
                if L.strip() == "" and i + 1 < n and lines[i + 1].startswith((" ", "\t")):
                    i += 1
                    continue
                break
            continue
        out.append(lines[i])
        i += 1
    return out


def hash_payload(doc: Dict[str, Any], seal_field: str = SEAL_FIELD) -> bytes:
    body_lf = normalize_lf(doc["body_raw"])
    kept = strip_reserved_fm_lines(doc["fm_lines"], seal_field)
    if any(l.strip() != "" for l in kept):
        payload = f"{INSTRUCT_V2_DOMAIN}\nFM\n" + "\n".join(kept) + f"\nBODY\n{body_lf}"
    else:
        payload = f"{INSTRUCT_V2_DOMAIN}\nBODY\n{body_lf}"
    return payload.encode("utf-8")


def compute_digest(doc: Dict[str, Any], seal_field: str = SEAL_FIELD) -> Dict[str, str]:
    return blake3_digest(hash_payload(doc, seal_field))


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


checkDocumentText = check_document_text
blake3Digest = blake3_digest
blake3Hex = blake3_hex

__all__ = [
    "SEAL_FIELD",
    "check_document_text",
    "checkDocumentText",
    "scan_suspicious_unicode",
    "blake3_digest",
    "blake3Digest",
    "blake3_hex",
    "blake3Hex",
]
