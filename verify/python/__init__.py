"""
@hashseal/verify — zero-dependency HashSeal instruct check (Python).
Signed, Sealed, Delivered — I'm Yours.
Copyright (c) 2026 MonkeyKing.dev
"""

from .check import (
    SEAL_FIELD,
    blake3Digest,
    blake3Hex,
    blake3_digest,
    blake3_hex,
    checkDocumentText,
    check_document_text,
)

__all__ = [
    "SEAL_FIELD",
    "check_document_text",
    "checkDocumentText",
    "blake3_digest",
    "blake3Digest",
    "blake3_hex",
    "blake3Hex",
]
