"""Minimal pure-Python RFC 8032 Ed25519 signature verification (stdlib only).

Copyright (c) 2026 MonkeyKing.dev
"""

from __future__ import annotations

import hashlib

_P = (1 << 255) - 19
_L = (1 << 252) + 27742317777372353535851937790883648493
_D = (-121665 * pow(121666, _P - 2, _P)) % _P
_I = pow(2, (_P - 1) // 4, _P)


def _inv(x: int) -> int:
    return pow(x, _P - 2, _P)


def _xrecover(y: int) -> int:
    xx = (y * y - 1) * _inv(_D * y * y + 1) % _P
    x = pow(xx, (_P + 3) // 8, _P)
    if (x * x - xx) % _P != 0:
        x = (x * _I) % _P
    if x % 2 != 0:
        x = _P - x
    return x


_GY = (4 * _inv(5)) % _P
_GX = _xrecover(_GY)
_G = (_GX, _GY, 1, (_GX * _GY) % _P)


def _edwards_add(p: tuple[int, int, int, int], q: tuple[int, int, int, int]) -> tuple[int, int, int, int]:
    x1, y1, z1, t1 = p
    x2, y2, z2, t2 = q
    a = ((y1 - x1) * (y2 - x2)) % _P
    b = ((y1 + x1) * (y2 + x2)) % _P
    c = (2 * _D * t1 * t2) % _P
    d = (2 * z1 * z2) % _P
    e = (b - a) % _P
    f = (d - c) % _P
    g = (d + c) % _P
    h = (b + a) % _P
    return ((e * f) % _P, (g * h) % _P, (f * g) % _P, (e * h) % _P)


def _scalar_mult(p: tuple[int, int, int, int], e: int) -> tuple[int, int, int, int]:
    q = (0, 1, 1, 0)
    cur = p
    while e > 0:
        if e & 1:
            q = _edwards_add(q, cur)
        cur = _edwards_add(cur, cur)
        e >>= 1
    return q


def _decode_point(b: bytes) -> tuple[int, int, int, int] | None:
    if len(b) != 32:
        return None
    y = int.from_bytes(b, "little")
    sign = y >> 255
    y &= (1 << 255) - 1
    if y >= _P:
        return None
    x = _xrecover(y)
    if (x & 1) != sign:
        x = _P - x
    if (-(x * x) + y * y - 1 - _D * x * x * y * y) % _P != 0:
        return None
    return (x, y, 1, (x * y) % _P)


def _point_eq(p: tuple[int, int, int, int], q: tuple[int, int, int, int]) -> bool:
    x1, y1, z1, _ = p
    x2, y2, z2, _ = q
    return (x1 * z2 - x2 * z1) % _P == 0 and (y1 * z2 - y2 * z1) % _P == 0


def verify_ed25519(pubkey: bytes, msg: bytes, sig: bytes) -> bool:
    if len(pubkey) != 32 or len(sig) != 64:
        return False
    a_pt = _decode_point(pubkey)
    r_pt = _decode_point(sig[:32])
    if a_pt is None or r_pt is None:
        return False
    s = int.from_bytes(sig[32:], "little")
    if s >= _L:
        return False
    h = int.from_bytes(hashlib.sha512(sig[:32] + pubkey + msg).digest(), "little") % _L
    lhs = _scalar_mult(_G, s)
    rhs = _edwards_add(r_pt, _scalar_mult(a_pt, h))
    return _point_eq(lhs, rhs)
