"""
In-memory tree verify — mirrors hashseal-core tree hash + verify policy.
Zero pip dependencies. Used for multi-lang tree-v1 vectors without a filesystem walk.

Copyright (c) 2026 MonkeyKing.dev
"""

from __future__ import annotations

from typing import Any, Dict, List, Mapping, Optional, Sequence, Set, Union

from check import blake3_digest

DEFAULT_TEXT_EXTENSIONS: frozenset[str] = frozenset(
    {
        "md",
        "txt",
        "toml",
        "yml",
        "yaml",
        "json",
        "rs",
        "java",
        "go",
        "py",
        "js",
        "ts",
        "tsx",
        "jsx",
        "css",
        "html",
        "xml",
        "sh",
        "ps1",
        "c",
        "h",
        "cpp",
        "cs",
        "rb",
        "svg",
    }
)


def normalize_lf(s: str) -> str:
    return s.replace("\r\n", "\n").replace("\r", "\n")


def ext_of(path: str) -> str:
    i = path.rfind(".")
    if i < 0:
        return ""
    return path[i + 1 :].lower()


def _text_ext_set(
    text_extensions: Optional[Union[Set[str], Sequence[str], frozenset[str]]],
) -> Set[str]:
    if text_extensions is None:
        return set(DEFAULT_TEXT_EXTENSIONS)
    if isinstance(text_extensions, set):
        return text_extensions
    return set(text_extensions)


def hash_tree_file_content(
    path: str,
    content: str,
    *,
    line_endings_lf_text: bool = True,
    text_extensions: Optional[Union[Set[str], Sequence[str], frozenset[str]]] = None,
) -> Dict[str, Any]:
    """Hash one path+content with core tree policy.

    size is on-disk UTF-8 byte length before normalize.
    """
    text_exts = _text_ext_set(text_extensions)
    size = len(content.encode("utf-8"))
    data = content
    if line_endings_lf_text and ext_of(path) in text_exts:
        if data and ord(data[0]) == 0xFEFF:
            data = data[1:]
        data = normalize_lf(data)
    d = blake3_digest(data)
    return {
        "digest": d["qualified"],
        "qualified": d["qualified"],
        "hex": d["hex"],
        "size": size,
    }


def verify_tree_in_memory(
    files: Optional[Mapping[str, str]],
    ledger_entries: Optional[Sequence[Mapping[str, Any]]],
    *,
    line_endings_lf_text: bool = True,
    text_extensions: Optional[Union[Set[str], Sequence[str], frozenset[str]]] = None,
) -> Dict[str, Any]:
    """Verify in-memory files against ledger entries (same findings as hashseal-core verify_tree)."""
    current: Dict[str, str] = {}
    for p in sorted((files or {}).keys()):
        h = hash_tree_file_content(
            p,
            files[p],  # type: ignore[index]
            line_endings_lf_text=line_endings_lf_text,
            text_extensions=text_extensions,
        )
        current[p] = h["qualified"]

    findings: List[Dict[str, Any]] = []
    expected_paths: Set[str] = set()
    entries = list(ledger_entries or [])

    for e in entries:
        path = e["path"]
        expected_paths.add(path)
        actual = current.get(path)
        if actual is None:
            findings.append(
                {
                    "path": path,
                    "status": "removed",
                    "expected": e["digest"],
                    "actual": None,
                }
            )
        elif actual != e["digest"]:
            findings.append(
                {
                    "path": path,
                    "status": "mismatch",
                    "expected": e["digest"],
                    "actual": actual,
                }
            )

    for path, digest in current.items():
        if path not in expected_paths:
            findings.append(
                {
                    "path": path,
                    "status": "added",
                    "expected": None,
                    "actual": digest,
                }
            )

    findings.sort(key=lambda f: f["path"])
    return {
        "ok": len(findings) == 0,
        "checked": len(entries),
        "findings": findings,
    }


# Public aliases matching JS SDK style
hashTreeFileContent = hash_tree_file_content
verifyTreeInMemory = verify_tree_in_memory

__all__ = [
    "DEFAULT_TEXT_EXTENSIONS",
    "normalize_lf",
    "hash_tree_file_content",
    "hashTreeFileContent",
    "verify_tree_in_memory",
    "verifyTreeInMemory",
]
