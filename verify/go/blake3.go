// Pure-Go BLAKE3 (hash mode) — port of official reference_impl / HashSeal Java vendor.
// Original BLAKE3 reference: CC0 1.0 / Apache-2.0.
// Copyright (c) 2026 MonkeyKing.dev (packaging)

package hashseal

const (
	outLen   = 32
	blockLen = 64
	chunkLen = 1024

	flagChunkStart = 1 << 0
	flagChunkEnd   = 1 << 1
	flagParent     = 1 << 2
	flagRoot       = 1 << 3
)

var iv = [8]uint32{
	0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
	0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
}

var msgPermutation = [16]int{2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8}

// Sum256 returns the 32-byte BLAKE3 hash of data.
func Sum256(data []byte) [32]byte {
	var h hasher
	h.init()
	h.update(data)
	out := h.digest(outLen)
	var sum [32]byte
	copy(sum[:], out)
	return sum
}

func rotr32(x uint32, n int) uint32 {
	return (x >> n) | (x << (32 - n))
}

func g(state *[16]uint32, a, b, c, d int, mx, my uint32) {
	state[a] = state[a] + state[b] + mx
	state[d] = rotr32(state[d]^state[a], 16)
	state[c] = state[c] + state[d]
	state[b] = rotr32(state[b]^state[c], 12)
	state[a] = state[a] + state[b] + my
	state[d] = rotr32(state[d]^state[a], 8)
	state[c] = state[c] + state[d]
	state[b] = rotr32(state[b]^state[c], 7)
}

func round(state *[16]uint32, m *[16]uint32) {
	g(state, 0, 4, 8, 12, m[0], m[1])
	g(state, 1, 5, 9, 13, m[2], m[3])
	g(state, 2, 6, 10, 14, m[4], m[5])
	g(state, 3, 7, 11, 15, m[6], m[7])
	g(state, 0, 5, 10, 15, m[8], m[9])
	g(state, 1, 6, 11, 12, m[10], m[11])
	g(state, 2, 7, 8, 13, m[12], m[13])
	g(state, 3, 4, 9, 14, m[14], m[15])
}

func permute(m *[16]uint32) {
	var p [16]uint32
	for i := 0; i < 16; i++ {
		p[i] = m[msgPermutation[i]]
	}
	*m = p
}

func compress(chainingValue *[8]uint32, blockWords *[16]uint32, counter uint64, blockLenU uint32, flags uint32) [16]uint32 {
	var state [16]uint32
	for i := 0; i < 8; i++ {
		state[i] = chainingValue[i]
	}
	state[8] = iv[0]
	state[9] = iv[1]
	state[10] = iv[2]
	state[11] = iv[3]
	state[12] = uint32(counter)
	state[13] = uint32(counter >> 32)
	state[14] = blockLenU
	state[15] = flags
	block := *blockWords
	for i := 0; i < 6; i++ {
		round(&state, &block)
		permute(&block)
	}
	round(&state, &block)
	for i := 0; i < 8; i++ {
		state[i] ^= state[i+8]
		state[i+8] ^= chainingValue[i]
	}
	return state
}

func first8(c [16]uint32) [8]uint32 {
	var o [8]uint32
	copy(o[:], c[:8])
	return o
}

func wordsFromLE(block []byte, words *[16]uint32) {
	for i := 0; i < 16; i++ {
		o := i * 4
		if o+3 >= len(block) {
			// pad with zeros already in block buffer
			var b [4]byte
			copy(b[:], block[o:])
			words[i] = uint32(b[0]) | uint32(b[1])<<8 | uint32(b[2])<<16 | uint32(b[3])<<24
		} else {
			words[i] = uint32(block[o]) | uint32(block[o+1])<<8 | uint32(block[o+2])<<16 | uint32(block[o+3])<<24
		}
	}
}

type output struct {
	inputCV    [8]uint32
	blockWords [16]uint32
	counter    uint64
	blockLen   uint32
	flags      uint32
}

func (o output) chainingValue() [8]uint32 {
	return first8(compress(&o.inputCV, &o.blockWords, o.counter, o.blockLen, o.flags))
}

func (o output) rootOutputBytes(outLen int) []byte {
	out := make([]byte, outLen)
	var outputBlockCounter uint64
	offset := 0
	for offset < outLen {
		words := compress(&o.inputCV, &o.blockWords, outputBlockCounter, o.blockLen, o.flags|flagRoot)
		for _, word := range words {
			take := 4
			if outLen-offset < take {
				take = outLen - offset
			}
			for b := 0; b < take; b++ {
				out[offset+b] = byte(word >> (8 * b))
			}
			offset += take
			if offset >= outLen {
				break
			}
		}
		outputBlockCounter++
	}
	return out
}

type chunkState struct {
	chainingValue   [8]uint32
	chunkCounter    uint64
	block           [blockLen]byte
	blockLen        int
	blocksCompressed int
	flags           uint32
}

func newChunkState(key [8]uint32, chunkCounter uint64, flags uint32) chunkState {
	return chunkState{chainingValue: key, chunkCounter: chunkCounter, flags: flags}
}

func (c *chunkState) len() int {
	return blockLen*c.blocksCompressed + c.blockLen
}

func (c *chunkState) startFlag() uint32 {
	if c.blocksCompressed == 0 {
		return flagChunkStart
	}
	return 0
}

func (c *chunkState) update(input []byte) {
	offset := 0
	for offset < len(input) {
		if c.blockLen == blockLen {
			var blockWords [16]uint32
			wordsFromLE(c.block[:], &blockWords)
			c.chainingValue = first8(compress(&c.chainingValue, &blockWords, c.chunkCounter, blockLen, c.flags|c.startFlag()))
			c.blocksCompressed++
			for i := range c.block {
				c.block[i] = 0
			}
			c.blockLen = 0
		}
		want := blockLen - c.blockLen
		take := want
		if len(input)-offset < take {
			take = len(input) - offset
		}
		copy(c.block[c.blockLen:], input[offset:offset+take])
		c.blockLen += take
		offset += take
	}
}

func (c *chunkState) output() output {
	var blockWords [16]uint32
	wordsFromLE(c.block[:], &blockWords)
	return output{
		inputCV:    c.chainingValue,
		blockWords: blockWords,
		counter:    c.chunkCounter,
		blockLen:   uint32(c.blockLen),
		flags:      c.flags | c.startFlag() | flagChunkEnd,
	}
}

func parentOutput(left, right, key [8]uint32, flags uint32) output {
	var blockWords [16]uint32
	copy(blockWords[:8], left[:])
	copy(blockWords[8:], right[:])
	return output{inputCV: key, blockWords: blockWords, counter: 0, blockLen: blockLen, flags: flagParent | flags}
}

func parentCV(left, right, key [8]uint32, flags uint32) [8]uint32 {
	return parentOutput(left, right, key, flags).chainingValue()
}

type hasher struct {
	chunkState chunkState
	keyWords   [8]uint32
	cvStack    [54][8]uint32
	cvStackLen int
	flags      uint32
}

func (h *hasher) init() {
	h.keyWords = iv
	h.flags = 0
	h.chunkState = newChunkState(h.keyWords, 0, h.flags)
	h.cvStackLen = 0
}

func (h *hasher) pushStack(cv [8]uint32) {
	h.cvStack[h.cvStackLen] = cv
	h.cvStackLen++
}

func (h *hasher) popStack() [8]uint32 {
	h.cvStackLen--
	return h.cvStack[h.cvStackLen]
}

func (h *hasher) addChunkChainingValue(newCV [8]uint32, totalChunks uint64) {
	for totalChunks&1 == 0 {
		newCV = parentCV(h.popStack(), newCV, h.keyWords, h.flags)
		totalChunks >>= 1
	}
	h.pushStack(newCV)
}

func (h *hasher) update(data []byte) {
	offset := 0
	for offset < len(data) {
		if h.chunkState.len() == chunkLen {
			chunkCV := h.chunkState.output().chainingValue()
			totalChunks := h.chunkState.chunkCounter + 1
			h.addChunkChainingValue(chunkCV, totalChunks)
			h.chunkState = newChunkState(h.keyWords, totalChunks, h.flags)
		}
		want := chunkLen - h.chunkState.len()
		take := want
		if len(data)-offset < take {
			take = len(data) - offset
		}
		h.chunkState.update(data[offset : offset+take])
		offset += take
	}
}

func (h *hasher) digest(outLen int) []byte {
	output := h.chunkState.output()
	parentNodesRemaining := h.cvStackLen
	for parentNodesRemaining > 0 {
		parentNodesRemaining--
		output = parentOutput(h.cvStack[parentNodesRemaining], output.chainingValue(), h.keyWords, h.flags)
	}
	return output.rootOutputBytes(outLen)
}
