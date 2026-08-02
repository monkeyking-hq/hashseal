// Pure C# BLAKE3 — port of the official BLAKE3 reference implementation.
// Source: https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs
// Original license: CC0 1.0 / Apache-2.0 (dual).
//
// Hash mode only. Zero NuGet dependencies (netstandard2.0 / net8.0 framework only).
//
// Copyright (c) 2026 MonkeyKing.dev (packaging)

using System;

namespace Hashseal.Verify
{
    /// <summary>Pure C# BLAKE3 hash mode.</summary>
    public static class Blake3
    {
        public const int OutLen = 32;
        public const int BlockLen = 64;
        public const int ChunkLen = 1024;

        private const int ChunkStart = 1 << 0;
        private const int ChunkEnd = 1 << 1;
        private const int Parent = 1 << 2;
        private const int Root = 1 << 3;

        private static readonly uint[] Iv =
        {
            0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
            0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19
        };

        private static readonly int[] MsgPermutation =
        {
            2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8
        };

        public static byte[] Hash(byte[] data)
        {
            return Hash(data, OutLen);
        }

        public static byte[] Hash(byte[] data, int outLen)
        {
            var h = new Hasher();
            h.Update(data);
            return h.Digest(outLen);
        }

        public static string HashHex(byte[] data)
        {
            return ToHex(Hash(data));
        }

        public static string ToHex(byte[] bytes)
        {
            char[] hex = new char[bytes.Length * 2];
            const string digits = "0123456789abcdef";
            for (int i = 0; i < bytes.Length; i++)
            {
                int v = bytes[i] & 0xff;
                hex[i * 2] = digits[v >> 4];
                hex[i * 2 + 1] = digits[v & 0x0f];
            }
            return new string(hex);
        }

        private static uint Rotr32(uint x, int n)
        {
            return (x >> n) | (x << (32 - n));
        }

        private static void G(uint[] state, int a, int b, int c, int d, uint mx, uint my)
        {
            state[a] = state[a] + state[b] + mx;
            state[d] = Rotr32(state[d] ^ state[a], 16);
            state[c] = state[c] + state[d];
            state[b] = Rotr32(state[b] ^ state[c], 12);
            state[a] = state[a] + state[b] + my;
            state[d] = Rotr32(state[d] ^ state[a], 8);
            state[c] = state[c] + state[d];
            state[b] = Rotr32(state[b] ^ state[c], 7);
        }

        private static void Round(uint[] state, uint[] m)
        {
            G(state, 0, 4, 8, 12, m[0], m[1]);
            G(state, 1, 5, 9, 13, m[2], m[3]);
            G(state, 2, 6, 10, 14, m[4], m[5]);
            G(state, 3, 7, 11, 15, m[6], m[7]);
            G(state, 0, 5, 10, 15, m[8], m[9]);
            G(state, 1, 6, 11, 12, m[10], m[11]);
            G(state, 2, 7, 8, 13, m[12], m[13]);
            G(state, 3, 4, 9, 14, m[14], m[15]);
        }

        private static void Permute(uint[] m)
        {
            uint[] permuted = new uint[16];
            for (int i = 0; i < 16; i++)
            {
                permuted[i] = m[MsgPermutation[i]];
            }
            Array.Copy(permuted, m, 16);
        }

        private static uint[] Compress(uint[] chainingValue, uint[] blockWords, ulong counter, uint blockLen, uint flags)
        {
            uint counterLow = (uint)counter;
            uint counterHigh = (uint)(counter >> 32);
            uint[] state = new uint[16];
            state[0] = chainingValue[0];
            state[1] = chainingValue[1];
            state[2] = chainingValue[2];
            state[3] = chainingValue[3];
            state[4] = chainingValue[4];
            state[5] = chainingValue[5];
            state[6] = chainingValue[6];
            state[7] = chainingValue[7];
            state[8] = Iv[0];
            state[9] = Iv[1];
            state[10] = Iv[2];
            state[11] = Iv[3];
            state[12] = counterLow;
            state[13] = counterHigh;
            state[14] = blockLen;
            state[15] = flags;
            uint[] block = new uint[16];
            Array.Copy(blockWords, block, 16);

            Round(state, block);
            Permute(block);
            Round(state, block);
            Permute(block);
            Round(state, block);
            Permute(block);
            Round(state, block);
            Permute(block);
            Round(state, block);
            Permute(block);
            Round(state, block);
            Permute(block);
            Round(state, block);

            for (int i = 0; i < 8; i++)
            {
                state[i] ^= state[i + 8];
                state[i + 8] ^= chainingValue[i];
            }
            return state;
        }

        private static uint[] First8(uint[] compressionOutput)
        {
            uint[] o = new uint[8];
            Array.Copy(compressionOutput, o, 8);
            return o;
        }

        private static void WordsFromLe(byte[] bytes, uint[] words)
        {
            for (int i = 0; i < words.Length; i++)
            {
                int o = i * 4;
                words[i] =
                    (uint)(bytes[o] & 0xff)
                    | ((uint)(bytes[o + 1] & 0xff) << 8)
                    | ((uint)(bytes[o + 2] & 0xff) << 16)
                    | ((uint)(bytes[o + 3] & 0xff) << 24);
            }
        }

        private sealed class Output
        {
            public readonly uint[] InputChainingValue;
            public readonly uint[] BlockWords;
            public readonly ulong Counter;
            public readonly uint BlockLenVal;
            public readonly uint Flags;

            public Output(uint[] inputChainingValue, uint[] blockWords, ulong counter, uint blockLen, uint flags)
            {
                InputChainingValue = inputChainingValue;
                BlockWords = blockWords;
                Counter = counter;
                BlockLenVal = blockLen;
                Flags = flags;
            }

            public uint[] ChainingValue()
            {
                return First8(Compress(InputChainingValue, BlockWords, Counter, BlockLenVal, Flags));
            }

            public byte[] RootOutputBytes(int outLen)
            {
                byte[] o = new byte[outLen];
                ulong outputBlockCounter = 0;
                int offset = 0;
                while (offset < outLen)
                {
                    uint[] words = Compress(
                        InputChainingValue, BlockWords, outputBlockCounter, BlockLenVal, Flags | Root);
                    foreach (uint word in words)
                    {
                        int take = Math.Min(4, outLen - offset);
                        for (int b = 0; b < take; b++)
                        {
                            o[offset + b] = (byte)((word >> (8 * b)) & 0xff);
                        }
                        offset += take;
                        if (offset >= outLen) break;
                    }
                    outputBlockCounter++;
                }
                return o;
            }
        }

        private sealed class ChunkState
        {
            public uint[] ChainingValue;
            public ulong ChunkCounter;
            public readonly byte[] Block = new byte[BlockLen];
            public int BlockLenVal;
            public int BlocksCompressed;
            public readonly uint Flags;

            public ChunkState(uint[] keyWords, ulong chunkCounter, uint flags)
            {
                ChainingValue = (uint[])keyWords.Clone();
                ChunkCounter = chunkCounter;
                Flags = flags;
            }

            public int Len()
            {
                return BlockLen * BlocksCompressed + BlockLenVal;
            }

            public uint StartFlag()
            {
                return BlocksCompressed == 0 ? (uint)ChunkStart : 0u;
            }

            public void Update(byte[] input, int off, int len)
            {
                int offset = off;
                int end = off + len;
                while (offset < end)
                {
                    if (BlockLenVal == BlockLen)
                    {
                        uint[] blockWords = new uint[16];
                        WordsFromLe(Block, blockWords);
                        ChainingValue = First8(
                            Compress(ChainingValue, blockWords, ChunkCounter, BlockLen, Flags | StartFlag()));
                        BlocksCompressed++;
                        Array.Clear(Block, 0, Block.Length);
                        BlockLenVal = 0;
                    }
                    int want = BlockLen - BlockLenVal;
                    int take = Math.Min(want, end - offset);
                    Array.Copy(input, offset, Block, BlockLenVal, take);
                    BlockLenVal += take;
                    offset += take;
                }
            }

            public Output Output()
            {
                uint[] blockWords = new uint[16];
                WordsFromLe(Block, blockWords);
                return new Output(
                    ChainingValue, blockWords, ChunkCounter, (uint)BlockLenVal,
                    Flags | StartFlag() | ChunkEnd);
            }
        }

        private static Output ParentOutput(uint[] leftChildCv, uint[] rightChildCv, uint[] keyWords, uint flags)
        {
            uint[] blockWords = new uint[16];
            Array.Copy(leftChildCv, 0, blockWords, 0, 8);
            Array.Copy(rightChildCv, 0, blockWords, 8, 8);
            return new Output(keyWords, blockWords, 0, BlockLen, Parent | flags);
        }

        private static uint[] ParentCv(uint[] leftChildCv, uint[] rightChildCv, uint[] keyWords, uint flags)
        {
            return ParentOutput(leftChildCv, rightChildCv, keyWords, flags).ChainingValue();
        }

        /// <summary>Incremental BLAKE3 hasher (regular hash mode).</summary>
        public sealed class Hasher
        {
            private ChunkState _chunkState;
            private readonly uint[] _keyWords;
            private readonly uint[][] _cvStack = new uint[54][];
            private int _cvStackLen;
            private readonly uint _flags;

            public Hasher()
            {
                _keyWords = (uint[])Iv.Clone();
                _flags = 0;
                _chunkState = new ChunkState(_keyWords, 0, _flags);
            }

            private void PushStack(uint[] cv)
            {
                _cvStack[_cvStackLen++] = cv;
            }

            private uint[] PopStack()
            {
                return _cvStack[--_cvStackLen];
            }

            private void AddChunkChainingValue(uint[] newCv, ulong totalChunks)
            {
                while ((totalChunks & 1) == 0)
                {
                    newCv = ParentCv(PopStack(), newCv, _keyWords, _flags);
                    totalChunks >>= 1;
                }
                PushStack(newCv);
            }

            public void Update(byte[] data)
            {
                Update(data, 0, data.Length);
            }

            public void Update(byte[] data, int off, int len)
            {
                int offset = off;
                int end = off + len;
                while (offset < end)
                {
                    if (_chunkState.Len() == ChunkLen)
                    {
                        uint[] chunkCv = _chunkState.Output().ChainingValue();
                        ulong totalChunks = _chunkState.ChunkCounter + 1;
                        AddChunkChainingValue(chunkCv, totalChunks);
                        _chunkState = new ChunkState(_keyWords, totalChunks, _flags);
                    }
                    int want = ChunkLen - _chunkState.Len();
                    int take = Math.Min(want, end - offset);
                    _chunkState.Update(data, offset, take);
                    offset += take;
                }
            }

            public byte[] Digest()
            {
                return Digest(OutLen);
            }

            public byte[] Digest(int outLen)
            {
                Output output = _chunkState.Output();
                int parentNodesRemaining = _cvStackLen;
                while (parentNodesRemaining > 0)
                {
                    parentNodesRemaining--;
                    output = ParentOutput(
                        _cvStack[parentNodesRemaining],
                        output.ChainingValue(),
                        _keyWords,
                        _flags);
                }
                return output.RootOutputBytes(outLen);
            }
        }
    }
}
