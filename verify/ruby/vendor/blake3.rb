# Pure-Ruby BLAKE3 — port of the official BLAKE3 reference implementation.
# Source: https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs
# Original license: CC0 1.0 / Apache-2.0 (dual).
#
# Hash mode only. Zero gem dependencies.
#
# Copyright (c) 2026 MonkeyKing.dev (packaging)

module Blake3
  OUT_LEN = 32
  BLOCK_LEN = 64
  CHUNK_LEN = 1024

  CHUNK_START = 1 << 0
  CHUNK_END = 1 << 1
  PARENT = 1 << 2
  ROOT = 1 << 3

  IV = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19
  ].freeze

  MSG_PERMUTATION = [2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8].freeze

  module_function

  def digest(data, out_len = OUT_LEN)
    h = Hasher.new
    h.update(data.b)
    h.digest(out_len)
  end

  def hexdigest(data, out_len = OUT_LEN)
    digest(data, out_len).unpack1("H*")
  end

  def rotr32(x, n)
    x &= 0xFFFFFFFF
    ((x >> n) | (x << (32 - n))) & 0xFFFFFFFF
  end

  def g(state, a, b, c, d, mx, my)
    state[a] = (state[a] + state[b] + mx) & 0xFFFFFFFF
    state[d] = rotr32(state[d] ^ state[a], 16)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = rotr32(state[b] ^ state[c], 12)
    state[a] = (state[a] + state[b] + my) & 0xFFFFFFFF
    state[d] = rotr32(state[d] ^ state[a], 8)
    state[c] = (state[c] + state[d]) & 0xFFFFFFFF
    state[b] = rotr32(state[b] ^ state[c], 7)
  end

  def round(state, m)
    g(state, 0, 4, 8, 12, m[0], m[1])
    g(state, 1, 5, 9, 13, m[2], m[3])
    g(state, 2, 6, 10, 14, m[4], m[5])
    g(state, 3, 7, 11, 15, m[6], m[7])
    g(state, 0, 5, 10, 15, m[8], m[9])
    g(state, 1, 6, 11, 12, m[10], m[11])
    g(state, 2, 7, 8, 13, m[12], m[13])
    g(state, 3, 4, 9, 14, m[14], m[15])
  end

  def permute!(m)
    permuted = MSG_PERMUTATION.map { |i| m[i] }
    16.times { |i| m[i] = permuted[i] }
  end

  def compress(chaining_value, block_words, counter, block_len, flags)
    counter_low = counter & 0xFFFFFFFF
    counter_high = (counter >> 32) & 0xFFFFFFFF
    state = [
      chaining_value[0], chaining_value[1], chaining_value[2], chaining_value[3],
      chaining_value[4], chaining_value[5], chaining_value[6], chaining_value[7],
      IV[0], IV[1], IV[2], IV[3],
      counter_low, counter_high, block_len & 0xFFFFFFFF, flags & 0xFFFFFFFF
    ]
    block = block_words.dup

    round(state, block)
    permute!(block)
    round(state, block)
    permute!(block)
    round(state, block)
    permute!(block)
    round(state, block)
    permute!(block)
    round(state, block)
    permute!(block)
    round(state, block)
    permute!(block)
    round(state, block)

    8.times do |i|
      state[i] = (state[i] ^ state[i + 8]) & 0xFFFFFFFF
      state[i + 8] = (state[i + 8] ^ chaining_value[i]) & 0xFFFFFFFF
    end
    state
  end

  def first8(compression_output)
    compression_output[0, 8]
  end

  def words_from_le(data)
    words = []
    i = 0
    while i < data.bytesize
      chunk = data.byteslice(i, 4)
      chunk = chunk.ljust(4, "\0") if chunk.bytesize < 4
      words << chunk.unpack1("V")
      i += 4
    end
    # pad to 16 words for partial last block (caller may pass full BLOCK_LEN)
    while words.length < 16
      words << 0
    end
    words[0, 16]
  end

  class Output
    attr_reader :input_chaining_value, :block_words, :counter, :block_len, :flags

    def initialize(input_chaining_value, block_words, counter, block_len, flags)
      @input_chaining_value = input_chaining_value
      @block_words = block_words
      @counter = counter
      @block_len = block_len
      @flags = flags
    end

    def chaining_value
      Blake3.first8(
        Blake3.compress(@input_chaining_value, @block_words, @counter, @block_len, @flags)
      )
    end

    def root_output_bytes(out_len)
      out = "".b
      output_block_counter = 0
      while out.bytesize < out_len
        words = Blake3.compress(
          @input_chaining_value, @block_words, output_block_counter, @block_len, @flags | ROOT
        )
        words.each do |word|
          chunk = [word].pack("V")
          take = [4, out_len - out.bytesize].min
          out << chunk.byteslice(0, take)
          break if out.bytesize >= out_len
        end
        output_block_counter += 1
      end
      out
    end
  end

  class ChunkState
    attr_accessor :chaining_value, :chunk_counter, :block, :block_len, :blocks_compressed, :flags

    def initialize(key_words, chunk_counter, flags)
      @chaining_value = key_words.dup
      @chunk_counter = chunk_counter
      @block = "\0".b * BLOCK_LEN
      @block_len = 0
      @blocks_compressed = 0
      @flags = flags
    end

    def len
      BLOCK_LEN * @blocks_compressed + @block_len
    end

    def start_flag
      @blocks_compressed.zero? ? CHUNK_START : 0
    end

    def update(data)
      data = data.b
      offset = 0
      while offset < data.bytesize
        if @block_len == BLOCK_LEN
          block_words = Blake3.words_from_le(@block)
          @chaining_value = Blake3.first8(
            Blake3.compress(
              @chaining_value, block_words, @chunk_counter, BLOCK_LEN, @flags | start_flag
            )
          )
          @blocks_compressed += 1
          @block = "\0".b * BLOCK_LEN
          @block_len = 0
        end
        want = BLOCK_LEN - @block_len
        take = [want, data.bytesize - offset].min
        @block[@block_len, take] = data.byteslice(offset, take)
        @block_len += take
        offset += take
      end
    end

    def output
      block_words = Blake3.words_from_le(@block)
      Output.new(
        @chaining_value, block_words, @chunk_counter, @block_len,
        @flags | start_flag | CHUNK_END
      )
    end
  end

  def self.parent_output(left_child_cv, right_child_cv, key_words, flags)
    block_words = left_child_cv + right_child_cv
    Output.new(key_words, block_words, 0, BLOCK_LEN, PARENT | flags)
  end

  def self.parent_cv(left_child_cv, right_child_cv, key_words, flags)
    parent_output(left_child_cv, right_child_cv, key_words, flags).chaining_value
  end

  class Hasher
    def initialize
      @key_words = IV.dup
      @flags = 0
      @chunk_state = ChunkState.new(@key_words, 0, @flags)
      @cv_stack = []
    end

    def update(data)
      data = data.b
      offset = 0
      while offset < data.bytesize
        if @chunk_state.len == CHUNK_LEN
          chunk_cv = @chunk_state.output.chaining_value
          total_chunks = @chunk_state.chunk_counter + 1
          add_chunk_chaining_value(chunk_cv, total_chunks)
          @chunk_state = ChunkState.new(@key_words, total_chunks, @flags)
        end
        want = CHUNK_LEN - @chunk_state.len
        take = [want, data.bytesize - offset].min
        @chunk_state.update(data.byteslice(offset, take))
        offset += take
      end
      self
    end

    def digest(length = OUT_LEN)
      output = @chunk_state.output
      parent_nodes_remaining = @cv_stack.length
      while parent_nodes_remaining > 0
        parent_nodes_remaining -= 1
        output = Blake3.parent_output(
          @cv_stack[parent_nodes_remaining],
          output.chaining_value,
          @key_words,
          @flags
        )
      end
      output.root_output_bytes(length)
    end

    def hexdigest(length = OUT_LEN)
      digest(length).unpack1("H*")
    end

    private

    def add_chunk_chaining_value(new_cv, total_chunks)
      while (total_chunks & 1).zero?
        new_cv = Blake3.parent_cv(@cv_stack.pop, new_cv, @key_words, @flags)
        total_chunks >>= 1
      end
      @cv_stack.push(new_cv)
    end
  end
end
