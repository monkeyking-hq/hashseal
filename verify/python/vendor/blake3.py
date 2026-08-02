# Pure-Python BLAKE3 — port of the official BLAKE3 reference implementation.
# Source: https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs
# Spec: https://github.com/BLAKE3-team/BLAKE3-specs
#
# License of original reference: CC0 1.0 Universal / Apache-2.0 (dual).
# This port is vendored for HashSeal zero-dep verify (no pip packages).
#
# Copyright note for this vendored file layout:
# Copyright (c) 2026 MonkeyKing.dev (packaging only)

"""Minimal pure-Python BLAKE3 (hash mode only)."""

from __future__ import annotations

from typing import List, Optional

OUT_LEN = 32
KEY_LEN = 32
BLOCK_LEN = 64
CHUNK_LEN = 1024

CHUNK_START = 1 << 0
CHUNK_END = 1 << 1
PARENT = 1 << 2
ROOT = 1 << 3
KEYED_HASH = 1 << 4
DERIVE_KEY_CONTEXT = 1 << 5
DERIVE_KEY_MATERIAL = 1 << 6

IV = [
    0x6A09E667,
    0xBB67AE85,
    0x3C6EF372,
    0xA54FF53A,
    0x510E527F,
    0x9B05688C,
    0x1F83D9AB,
    0x5BE0CD19,
]

MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8]


def _rotr32(x: int, n: int) -> int:
    x &= 0xFFFFFFFF
    return ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF


def _g(state: List[int], a: int, b: int, c: int, d: int, mx: int, my: int) -> None:
    state[a] = (state[a] + state[b] + mx) & 0xFFFFFFFF
    state[d] = _rotr32(state[d] ^ state[a], 16)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = _rotr32(state[b] ^ state[c], 12)
    state[a] = (state[a] + state[b] + my) & 0xFFFFFFFF
    state[d] = _rotr32(state[d] ^ state[a], 8)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = _rotr32(state[b] ^ state[c], 7)


def _round(state: List[int], m: List[int]) -> None:
    _g(state, 0, 4, 8, 12, m[0], m[1])
    _g(state, 1, 5, 9, 13, m[2], m[3])
    _g(state, 2, 6, 10, 14, m[4], m[5])
    _g(state, 3, 7, 11, 15, m[6], m[7])
    _g(state, 0, 5, 10, 15, m[8], m[9])
    _g(state, 1, 6, 11, 12, m[10], m[11])
    _g(state, 2, 7, 8, 13, m[12], m[13])
    _g(state, 3, 4, 9, 14, m[14], m[15])


def _permute(m: List[int]) -> None:
    permuted = [m[MSG_PERMUTATION[i]] for i in range(16)]
    for i in range(16):
        m[i] = permuted[i]


def _compress(
    chaining_value: List[int],
    block_words: List[int],
    counter: int,
    block_len: int,
    flags: int,
) -> List[int]:
    counter_low = counter & 0xFFFFFFFF
    counter_high = (counter >> 32) & 0xFFFFFFFF
    state = [
        chaining_value[0],
        chaining_value[1],
        chaining_value[2],
        chaining_value[3],
        chaining_value[4],
        chaining_value[5],
        chaining_value[6],
        chaining_value[7],
        IV[0],
        IV[1],
        IV[2],
        IV[3],
        counter_low,
        counter_high,
        block_len & 0xFFFFFFFF,
        flags & 0xFFFFFFFF,
    ]
    block = list(block_words)

    _round(state, block)
    _permute(block)
    _round(state, block)
    _permute(block)
    _round(state, block)
    _permute(block)
    _round(state, block)
    _permute(block)
    _round(state, block)
    _permute(block)
    _round(state, block)
    _permute(block)
    _round(state, block)

    for i in range(8):
        state[i] = (state[i] ^ state[i + 8]) & 0xFFFFFFFF
        state[i + 8] = (state[i + 8] ^ chaining_value[i]) & 0xFFFFFFFF
    return state


def _first_8_words(compression_output: List[int]) -> List[int]:
    return compression_output[0:8]


def _words_from_le_bytes(data: bytes) -> List[int]:
    words: List[int] = []
    for i in range(0, len(data), 4):
        words.append(int.from_bytes(data[i : i + 4], "little"))
    return words


class _Output:
    __slots__ = (
        "input_chaining_value",
        "block_words",
        "counter",
        "block_len",
        "flags",
    )

    def __init__(
        self,
        input_chaining_value: List[int],
        block_words: List[int],
        counter: int,
        block_len: int,
        flags: int,
    ) -> None:
        self.input_chaining_value = input_chaining_value
        self.block_words = block_words
        self.counter = counter
        self.block_len = block_len
        self.flags = flags

    def chaining_value(self) -> List[int]:
        return _first_8_words(
            _compress(
                self.input_chaining_value,
                self.block_words,
                self.counter,
                self.block_len,
                self.flags,
            )
        )

    def root_output_bytes(self, out_len: int) -> bytes:
        out = bytearray(out_len)
        output_block_counter = 0
        offset = 0
        while offset < out_len:
            words = _compress(
                self.input_chaining_value,
                self.block_words,
                output_block_counter,
                self.block_len,
                self.flags | ROOT,
            )
            for word in words:
                chunk = word.to_bytes(4, "little")
                take = min(4, out_len - offset)
                out[offset : offset + take] = chunk[:take]
                offset += take
                if offset >= out_len:
                    break
            output_block_counter += 1
        return bytes(out)


class _ChunkState:
    __slots__ = (
        "chaining_value",
        "chunk_counter",
        "block",
        "block_len",
        "blocks_compressed",
        "flags",
    )

    def __init__(self, key_words: List[int], chunk_counter: int, flags: int) -> None:
        self.chaining_value = list(key_words)
        self.chunk_counter = chunk_counter
        self.block = bytearray(BLOCK_LEN)
        self.block_len = 0
        self.blocks_compressed = 0
        self.flags = flags

    def len(self) -> int:
        return BLOCK_LEN * self.blocks_compressed + self.block_len

    def start_flag(self) -> int:
        return CHUNK_START if self.blocks_compressed == 0 else 0

    def update(self, data: bytes) -> None:
        offset = 0
        while offset < len(data):
            if self.block_len == BLOCK_LEN:
                block_words = _words_from_le_bytes(bytes(self.block))
                self.chaining_value = _first_8_words(
                    _compress(
                        self.chaining_value,
                        block_words,
                        self.chunk_counter,
                        BLOCK_LEN,
                        self.flags | self.start_flag(),
                    )
                )
                self.blocks_compressed += 1
                self.block = bytearray(BLOCK_LEN)
                self.block_len = 0
            want = BLOCK_LEN - self.block_len
            take = min(want, len(data) - offset)
            self.block[self.block_len : self.block_len + take] = data[offset : offset + take]
            self.block_len += take
            offset += take

    def output(self) -> _Output:
        block_words = _words_from_le_bytes(bytes(self.block))
        return _Output(
            self.chaining_value,
            block_words,
            self.chunk_counter,
            self.block_len,
            self.flags | self.start_flag() | CHUNK_END,
        )


def _parent_output(
    left_child_cv: List[int],
    right_child_cv: List[int],
    key_words: List[int],
    flags: int,
) -> _Output:
    block_words = list(left_child_cv) + list(right_child_cv)
    return _Output(key_words, block_words, 0, BLOCK_LEN, PARENT | flags)


def _parent_cv(
    left_child_cv: List[int],
    right_child_cv: List[int],
    key_words: List[int],
    flags: int,
) -> List[int]:
    return _parent_output(left_child_cv, right_child_cv, key_words, flags).chaining_value()


class Hasher:
    """Incremental BLAKE3 hasher (regular hash mode)."""

    __slots__ = ("chunk_state", "key_words", "cv_stack", "flags")

    def __init__(self, key_words: Optional[List[int]] = None, flags: int = 0) -> None:
        self.key_words = list(key_words) if key_words is not None else list(IV)
        self.flags = flags
        self.chunk_state = _ChunkState(self.key_words, 0, flags)
        self.cv_stack: List[List[int]] = []

    def _push_stack(self, cv: List[int]) -> None:
        self.cv_stack.append(cv)

    def _pop_stack(self) -> List[int]:
        return self.cv_stack.pop()

    def _add_chunk_chaining_value(self, new_cv: List[int], total_chunks: int) -> None:
        while total_chunks & 1 == 0:
            new_cv = _parent_cv(self._pop_stack(), new_cv, self.key_words, self.flags)
            total_chunks >>= 1
        self._push_stack(new_cv)

    def update(self, data: bytes) -> "Hasher":
        offset = 0
        n = len(data)
        while offset < n:
            if self.chunk_state.len() == CHUNK_LEN:
                chunk_cv = self.chunk_state.output().chaining_value()
                total_chunks = self.chunk_state.chunk_counter + 1
                self._add_chunk_chaining_value(chunk_cv, total_chunks)
                self.chunk_state = _ChunkState(self.key_words, total_chunks, self.flags)
            want = CHUNK_LEN - self.chunk_state.len()
            take = min(want, n - offset)
            self.chunk_state.update(data[offset : offset + take])
            offset += take
        return self

    def digest(self, length: int = OUT_LEN) -> bytes:
        output = self.chunk_state.output()
        parent_nodes_remaining = len(self.cv_stack)
        while parent_nodes_remaining > 0:
            parent_nodes_remaining -= 1
            output = _parent_output(
                self.cv_stack[parent_nodes_remaining],
                output.chaining_value(),
                self.key_words,
                self.flags,
            )
        return output.root_output_bytes(length)

    def hexdigest(self, length: int = OUT_LEN) -> str:
        return self.digest(length).hex()


def blake3(data: bytes, length: int = OUT_LEN) -> bytes:
    """One-shot BLAKE3 hash of `data`."""
    return Hasher().update(data).digest(length)


def blake3_hex(data: bytes) -> str:
    return blake3(data).hex()
