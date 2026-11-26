"""Pure-Python OpenSSH SSHSIG parser + Ed25519 verifier (stdlib only).

Copyright (c) 2026 MonkeyKing.dev
"""

from __future__ import annotations

import base64
import hashlib
import struct
import sys
from pathlib import Path
from typing import Any, Dict, List, Tuple

_VENDOR = Path(__file__).resolve().parent / "vendor"
if str(_VENDOR) not in sys.path:
    sys.path.insert(0, str(_VENDOR))

from ed25519 import verify_ed25519  # type: ignore  # noqa: E402


class _Reader:
    def __init__(self, data: bytes) -> None:
        self.data = data
        self.off = 0

    def remaining(self) -> int:
        return len(self.data) - self.off

    def read_bytes(self, n: int) -> bytes:
        if self.off + n > len(self.data):
            raise ValueError("unexpected EOF in SSHSIG")
        out = self.data[self.off : self.off + n]
        self.off += n
        return out

    def read_u32(self) -> int:
        return struct.unpack(">I", self.read_bytes(4))[0]

    def read_string(self) -> bytes:
        return self.read_bytes(self.read_u32())


def _ssh_string(b: bytes | str) -> bytes:
    raw = b.encode("utf-8") if isinstance(b, str) else b
    return struct.pack(">I", len(raw)) + raw


def parse_sshsig(armor: str) -> Dict[str, Any]:
    lines = [ln.strip() for ln in armor.replace("\r\n", "\n").split("\n")]
    try:
        start = lines.index("-----BEGIN SSH SIGNATURE-----")
        end = lines.index("-----END SSH SIGNATURE-----")
    except ValueError as exc:
        raise ValueError("invalid SSH SIGNATURE armor") from exc
    if end <= start:
        raise ValueError("invalid SSH SIGNATURE armor")
    b64 = "".join(lines[start + 1 : end])
    if not b64:
        raise ValueError("empty SSH SIGNATURE armor")
    raw = base64.b64decode(b64, validate=True)

    r = _Reader(raw)
    if r.read_bytes(6) != b"SSHSIG":
        raise ValueError("bad SSHSIG magic")
    version = r.read_u32()
    if version != 1:
        raise ValueError(f"unsupported SSHSIG version {version}")
    pub_blob = r.read_string()
    namespace = r.read_string().decode("utf-8")
    reserved = r.read_string()
    hash_alg = r.read_string().decode("utf-8")
    sig_blob = r.read_string()
    if r.remaining() != 0:
        raise ValueError("trailing bytes in SSHSIG")

    pr = _Reader(pub_blob)
    pub_type = pr.read_string().decode("utf-8")
    if pub_type != "ssh-ed25519":
        raise ValueError(f"unsupported SSH key type: {pub_type}")
    pub_key = pr.read_string()
    if len(pub_key) != 32:
        raise ValueError("invalid Ed25519 public key length")

    sr = _Reader(sig_blob)
    sig_type = sr.read_string().decode("utf-8")
    if sig_type != "ssh-ed25519":
        raise ValueError(f"unsupported SSH sig type: {sig_type}")
    sig_bytes = sr.read_string()
    if len(sig_bytes) != 64:
        raise ValueError("invalid Ed25519 signature length")

    return {
        "pub_blob": pub_blob,
        "pub_key": pub_key,
        "namespace": namespace,
        "reserved": reserved,
        "hash_alg": hash_alg,
        "sig_bytes": sig_bytes,
    }


def parse_allowed_signers(text: str) -> List[Tuple[str, bytes]]:
    entries: List[Tuple[str, bytes]] = []
    for raw_line in text.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split()
        if len(parts) < 3:
            continue
        principals = [p.strip() for p in parts[0].split(",") if p.strip()]
        idx = 1
        if not parts[idx].startswith(("ssh-", "ecdsa-", "sk-")):
            idx += 1
        if idx + 1 >= len(parts):
            continue
        key_type = parts[idx]
        b64 = parts[idx + 1]
        if key_type != "ssh-ed25519":
            continue
        try:
            key_blob = base64.b64decode(b64, validate=True)
        except Exception:
            continue
        for p in principals:
            entries.append((p, key_blob))
    return entries


def verify_sshsig(
    payload: bytes | str,
    armor: str,
    allowed_signers_text: str,
    namespace: str = "hashseal",
) -> Dict[str, Any]:
    try:
        parsed = parse_sshsig(armor)
    except Exception as exc:
        return {"status": "bad", "message": str(exc)}

    if parsed["namespace"] != namespace:
        return {
            "status": "bad",
            "message": f"namespace mismatch: got {parsed['namespace']}, want {namespace}",
        }

    signers = parse_allowed_signers(allowed_signers_text)
    matching = [p for p, blob in signers if blob == parsed["pub_blob"]]
    if not matching:
        return {"status": "untrusted_key", "message": "signer key not in allowed_signers"}

    data = payload.encode("utf-8") if isinstance(payload, str) else payload
    if parsed["hash_alg"] == "sha512":
        digest = hashlib.sha512(data).digest()
    elif parsed["hash_alg"] == "sha256":
        digest = hashlib.sha256(data).digest()
    else:
        return {"status": "bad", "message": f"unsupported SSHSIG hash: {parsed['hash_alg']}"}

    signed_data = (
        b"SSHSIG"
        + _ssh_string(parsed["namespace"])
        + _ssh_string(parsed["reserved"])
        + _ssh_string(parsed["hash_alg"])
        + _ssh_string(digest)
    )
    if not verify_ed25519(parsed["pub_key"], signed_data, parsed["sig_bytes"]):
        return {"status": "bad", "message": "Ed25519 SSHSIG verification failed"}
    return {"status": "good", "signer": matching[0]}
