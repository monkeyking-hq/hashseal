package ai.hashseal.verify;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;

/**
 * In-memory tree verify — mirrors hashseal-core tree hash + verify policy.
 * Zero Maven dependencies. Used for multi-lang tree-v1 vectors.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
public final class Tree {
  private Tree() {}

  /** Default text extensions (LF + BOM strip policy). */
  public static final Set<String> DEFAULT_TEXT_EXTENSIONS;

  static {
    Set<String> s = new HashSet<>();
    Collections.addAll(
        s,
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
        "svg");
    DEFAULT_TEXT_EXTENSIONS = Collections.unmodifiableSet(s);
  }

  /** Result of hashing one path under tree policy. */
  public static final class FileHash {
    public final String digest;
    public final String qualified;
    public final String hex;
    /** On-disk UTF-8 byte length before normalize. */
    public final int size;

    public FileHash(String qualified, String hex, int size) {
      this.digest = qualified;
      this.qualified = qualified;
      this.hex = hex;
      this.size = size;
    }
  }

  /** One non-OK path from tree verify. */
  public static final class Finding {
    public final String path;
    public final String status; // mismatch | removed | added
    public final String expected;
    public final String actual;

    public Finding(String path, String status, String expected, String actual) {
      this.path = path;
      this.status = status;
      this.expected = expected;
      this.actual = actual;
    }
  }

  /** Result of {@link #verifyTreeInMemory}. */
  public static final class VerifyResult {
    public final boolean ok;
    public final int checked;
    public final List<Finding> findings;

    public VerifyResult(boolean ok, int checked, List<Finding> findings) {
      this.ok = ok;
      this.checked = checked;
      this.findings = findings;
    }
  }

  /** Ledger row. */
  public static final class LedgerEntry {
    public final String path;
    public final String digest;
    public final int size;

    public LedgerEntry(String path, String digest, int size) {
      this.path = path;
      this.digest = digest;
      this.size = size;
    }
  }

  public static String normalizeLf(String s) {
    return s.replace("\r\n", "\n").replace("\r", "\n");
  }

  static String extOf(String path) {
    int i = path.lastIndexOf('.');
    if (i < 0) {
      return "";
    }
    return path.substring(i + 1).toLowerCase();
  }

  /**
   * Hash one path+content with core tree policy.
   *
   * @param lineEndingsLfText when true, text extensions get BOM strip + LF normalize
   * @param textExtensions null → {@link #DEFAULT_TEXT_EXTENSIONS}
   */
  public static FileHash hashTreeFileContent(
      String path, String content, boolean lineEndingsLfText, Set<String> textExtensions) {
    Set<String> textExts = textExtensions != null ? textExtensions : DEFAULT_TEXT_EXTENSIONS;
    int size = content.getBytes(StandardCharsets.UTF_8).length;
    String data = content;
    if (lineEndingsLfText && textExts.contains(extOf(path))) {
      if (!data.isEmpty() && data.charAt(0) == '\uFEFF') {
        data = data.substring(1);
      }
      data = normalizeLf(data);
    }
    Check.Digest d = Check.blake3Digest(data);
    return new FileHash(d.qualified, d.hex, size);
  }

  public static FileHash hashTreeFileContent(String path, String content) {
    return hashTreeFileContent(path, content, true, null);
  }

  /**
   * Verify in-memory files against ledger entries (same findings as core verify_tree).
   * Files map is treated as path → content; iteration order does not matter.
   */
  public static VerifyResult verifyTreeInMemory(
      Map<String, String> files,
      List<LedgerEntry> ledgerEntries,
      boolean lineEndingsLfText,
      Set<String> textExtensions) {
    if (files == null) {
      files = Collections.emptyMap();
    }
    // TreeMap for sorted path order when hashing / listing added
    Map<String, String> sortedFiles = new TreeMap<>(files);
    Map<String, String> current = new LinkedHashMap<>();
    for (Map.Entry<String, String> e : sortedFiles.entrySet()) {
      FileHash h =
          hashTreeFileContent(e.getKey(), e.getValue(), lineEndingsLfText, textExtensions);
      current.put(e.getKey(), h.qualified);
    }

    List<Finding> findings = new ArrayList<>();
    Set<String> expectedPaths = new HashSet<>();
    List<LedgerEntry> entries =
        ledgerEntries != null ? ledgerEntries : Collections.emptyList();

    for (LedgerEntry e : entries) {
      expectedPaths.add(e.path);
      String actual = current.get(e.path);
      if (actual == null) {
        findings.add(new Finding(e.path, "removed", e.digest, null));
      } else if (!actual.equals(e.digest)) {
        findings.add(new Finding(e.path, "mismatch", e.digest, actual));
      }
    }

    for (Map.Entry<String, String> e : current.entrySet()) {
      if (!expectedPaths.contains(e.getKey())) {
        findings.add(new Finding(e.getKey(), "added", null, e.getValue()));
      }
    }

    findings.sort((a, b) -> a.path.compareTo(b.path));
    return new VerifyResult(findings.isEmpty(), entries.size(), findings);
  }

  public static VerifyResult verifyTreeInMemory(
      Map<String, String> files, List<LedgerEntry> ledgerEntries) {
    return verifyTreeInMemory(files, ledgerEntries, true, null);
  }

  /** Build a mutable extension set from a list (vector text_extensions). */
  public static Set<String> textExtensionsFromList(List<String> exts) {
    Set<String> s = new HashSet<>();
    if (exts != null) {
      for (String e : exts) {
        s.add(e.toLowerCase());
      }
    }
    return s;
  }
}
