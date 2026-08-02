package ai.hashseal.verify;

/**
 * Pure-Java BLAKE3 — port of the official BLAKE3 reference implementation.
 * Source: https://github.com/BLAKE3-team/BLAKE3/blob/master/reference_impl/reference_impl.rs
 * Original license: CC0 1.0 / Apache-2.0 (dual).
 *
 * Hash mode only. No external Maven dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev (packaging)
 */
public final class Blake3 {
  public static final int OUT_LEN = 32;
  public static final int BLOCK_LEN = 64;
  public static final int CHUNK_LEN = 1024;

  private static final int CHUNK_START = 1 << 0;
  private static final int CHUNK_END = 1 << 1;
  private static final int PARENT = 1 << 2;
  private static final int ROOT = 1 << 3;

  private static final int[] IV = {
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A,
    0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19
  };

  private static final int[] MSG_PERMUTATION = {
    2, 6, 3, 10, 7, 0, 4, 13, 1, 11, 12, 5, 9, 14, 15, 8
  };

  private Blake3() {}

  public static byte[] hash(byte[] data) {
    return hash(data, OUT_LEN);
  }

  public static byte[] hash(byte[] data, int outLen) {
    Hasher h = new Hasher();
    h.update(data);
    return h.digest(outLen);
  }

  public static String hashHex(byte[] data) {
    return toHex(hash(data));
  }

  public static String toHex(byte[] bytes) {
    char[] hex = new char[bytes.length * 2];
    final char[] digits = "0123456789abcdef".toCharArray();
    for (int i = 0; i < bytes.length; i++) {
      int v = bytes[i] & 0xff;
      hex[i * 2] = digits[v >>> 4];
      hex[i * 2 + 1] = digits[v & 0x0f];
    }
    return new String(hex);
  }

  private static int rotr32(int x, int n) {
    return (x >>> n) | (x << (32 - n));
  }

  private static void g(int[] state, int a, int b, int c, int d, int mx, int my) {
    state[a] = state[a] + state[b] + mx;
    state[d] = rotr32(state[d] ^ state[a], 16);
    state[c] = state[c] + state[d];
    state[b] = rotr32(state[b] ^ state[c], 12);
    state[a] = state[a] + state[b] + my;
    state[d] = rotr32(state[d] ^ state[a], 8);
    state[c] = state[c] + state[d];
    state[b] = rotr32(state[b] ^ state[c], 7);
  }

  private static void round(int[] state, int[] m) {
    g(state, 0, 4, 8, 12, m[0], m[1]);
    g(state, 1, 5, 9, 13, m[2], m[3]);
    g(state, 2, 6, 10, 14, m[4], m[5]);
    g(state, 3, 7, 11, 15, m[6], m[7]);
    g(state, 0, 5, 10, 15, m[8], m[9]);
    g(state, 1, 6, 11, 12, m[10], m[11]);
    g(state, 2, 7, 8, 13, m[12], m[13]);
    g(state, 3, 4, 9, 14, m[14], m[15]);
  }

  private static void permute(int[] m) {
    int[] permuted = new int[16];
    for (int i = 0; i < 16; i++) {
      permuted[i] = m[MSG_PERMUTATION[i]];
    }
    System.arraycopy(permuted, 0, m, 0, 16);
  }

  private static int[] compress(
      int[] chainingValue, int[] blockWords, long counter, int blockLen, int flags) {
    int counterLow = (int) counter;
    int counterHigh = (int) (counter >>> 32);
    int[] state = new int[16];
    state[0] = chainingValue[0];
    state[1] = chainingValue[1];
    state[2] = chainingValue[2];
    state[3] = chainingValue[3];
    state[4] = chainingValue[4];
    state[5] = chainingValue[5];
    state[6] = chainingValue[6];
    state[7] = chainingValue[7];
    state[8] = IV[0];
    state[9] = IV[1];
    state[10] = IV[2];
    state[11] = IV[3];
    state[12] = counterLow;
    state[13] = counterHigh;
    state[14] = blockLen;
    state[15] = flags;
    int[] block = new int[16];
    System.arraycopy(blockWords, 0, block, 0, 16);

    round(state, block);
    permute(block);
    round(state, block);
    permute(block);
    round(state, block);
    permute(block);
    round(state, block);
    permute(block);
    round(state, block);
    permute(block);
    round(state, block);
    permute(block);
    round(state, block);

    for (int i = 0; i < 8; i++) {
      state[i] ^= state[i + 8];
      state[i + 8] ^= chainingValue[i];
    }
    return state;
  }

  private static int[] first8(int[] compressionOutput) {
    int[] out = new int[8];
    System.arraycopy(compressionOutput, 0, out, 0, 8);
    return out;
  }

  private static void wordsFromLe(byte[] bytes, int[] words) {
    for (int i = 0; i < words.length; i++) {
      int o = i * 4;
      words[i] =
          (bytes[o] & 0xff)
              | ((bytes[o + 1] & 0xff) << 8)
              | ((bytes[o + 2] & 0xff) << 16)
              | ((bytes[o + 3] & 0xff) << 24);
    }
  }

  private static final class Output {
    final int[] inputChainingValue;
    final int[] blockWords;
    final long counter;
    final int blockLen;
    final int flags;

    Output(int[] inputChainingValue, int[] blockWords, long counter, int blockLen, int flags) {
      this.inputChainingValue = inputChainingValue;
      this.blockWords = blockWords;
      this.counter = counter;
      this.blockLen = blockLen;
      this.flags = flags;
    }

    int[] chainingValue() {
      return first8(compress(inputChainingValue, blockWords, counter, blockLen, flags));
    }

    byte[] rootOutputBytes(int outLen) {
      byte[] out = new byte[outLen];
      long outputBlockCounter = 0;
      int offset = 0;
      while (offset < outLen) {
        int[] words =
            compress(inputChainingValue, blockWords, outputBlockCounter, blockLen, flags | ROOT);
        for (int word : words) {
          int take = Math.min(4, outLen - offset);
          for (int b = 0; b < take; b++) {
            out[offset + b] = (byte) ((word >>> (8 * b)) & 0xff);
          }
          offset += take;
          if (offset >= outLen) {
            break;
          }
        }
        outputBlockCounter++;
      }
      return out;
    }
  }

  private static final class ChunkState {
    int[] chainingValue;
    long chunkCounter;
    final byte[] block = new byte[BLOCK_LEN];
    int blockLen;
    int blocksCompressed;
    final int flags;

    ChunkState(int[] keyWords, long chunkCounter, int flags) {
      this.chainingValue = keyWords.clone();
      this.chunkCounter = chunkCounter;
      this.flags = flags;
    }

    int len() {
      return BLOCK_LEN * blocksCompressed + blockLen;
    }

    int startFlag() {
      return blocksCompressed == 0 ? CHUNK_START : 0;
    }

    void update(byte[] input, int off, int len) {
      int offset = off;
      int end = off + len;
      while (offset < end) {
        if (blockLen == BLOCK_LEN) {
          int[] blockWords = new int[16];
          wordsFromLe(block, blockWords);
          chainingValue =
              first8(
                  compress(
                      chainingValue, blockWords, chunkCounter, BLOCK_LEN, flags | startFlag()));
          blocksCompressed++;
          java.util.Arrays.fill(block, (byte) 0);
          blockLen = 0;
        }
        int want = BLOCK_LEN - blockLen;
        int take = Math.min(want, end - offset);
        System.arraycopy(input, offset, block, blockLen, take);
        blockLen += take;
        offset += take;
      }
    }

    Output output() {
      int[] blockWords = new int[16];
      wordsFromLe(block, blockWords);
      return new Output(
          chainingValue, blockWords, chunkCounter, blockLen, flags | startFlag() | CHUNK_END);
    }
  }

  private static Output parentOutput(
      int[] leftChildCv, int[] rightChildCv, int[] keyWords, int flags) {
    int[] blockWords = new int[16];
    System.arraycopy(leftChildCv, 0, blockWords, 0, 8);
    System.arraycopy(rightChildCv, 0, blockWords, 8, 8);
    return new Output(keyWords, blockWords, 0, BLOCK_LEN, PARENT | flags);
  }

  private static int[] parentCv(
      int[] leftChildCv, int[] rightChildCv, int[] keyWords, int flags) {
    return parentOutput(leftChildCv, rightChildCv, keyWords, flags).chainingValue();
  }

  /** Incremental BLAKE3 hasher (regular hash mode). */
  public static final class Hasher {
    private ChunkState chunkState;
    private final int[] keyWords;
    private final int[][] cvStack = new int[54][];
    private int cvStackLen;
    private final int flags;

    public Hasher() {
      this.keyWords = IV.clone();
      this.flags = 0;
      this.chunkState = new ChunkState(keyWords, 0, flags);
    }

    private void pushStack(int[] cv) {
      cvStack[cvStackLen++] = cv;
    }

    private int[] popStack() {
      return cvStack[--cvStackLen];
    }

    private void addChunkChainingValue(int[] newCv, long totalChunks) {
      while ((totalChunks & 1) == 0) {
        newCv = parentCv(popStack(), newCv, keyWords, flags);
        totalChunks >>>= 1;
      }
      pushStack(newCv);
    }

    public void update(byte[] data) {
      update(data, 0, data.length);
    }

    public void update(byte[] data, int off, int len) {
      int offset = off;
      int end = off + len;
      while (offset < end) {
        if (chunkState.len() == CHUNK_LEN) {
          int[] chunkCv = chunkState.output().chainingValue();
          long totalChunks = chunkState.chunkCounter + 1;
          addChunkChainingValue(chunkCv, totalChunks);
          chunkState = new ChunkState(keyWords, totalChunks, flags);
        }
        int want = CHUNK_LEN - chunkState.len();
        int take = Math.min(want, end - offset);
        chunkState.update(data, offset, take);
        offset += take;
      }
    }

    public byte[] digest() {
      return digest(OUT_LEN);
    }

    public byte[] digest(int outLen) {
      Output output = chunkState.output();
      int parentNodesRemaining = cvStackLen;
      while (parentNodesRemaining > 0) {
        parentNodesRemaining--;
        output =
            parentOutput(
                cvStack[parentNodesRemaining],
                output.chainingValue(),
                keyWords,
                flags);
      }
      return output.rootOutputBytes(outLen);
    }
  }
}
