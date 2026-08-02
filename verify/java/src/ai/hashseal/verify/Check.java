package ai.hashseal.verify;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import java.util.function.BiConsumer;

/**
 * HashSeal instruct document check — FULL canonical mode (digest only).
 * Mirrors hashseal-core instruct algorithm. Zero Maven dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
public final class Check {
  public static final String SEAL_FIELD = "hashseal";
  public static final String SIG_FIELD = "hashseal_sig";
  public static final String KEY_ID_FIELD = "hashseal_key_id";

  private Check() {}

  /** Result of {@link #checkDocumentText(String)}. */
  public static final class Result {
    public final boolean ok;
    public final String status;
    public final String algorithm;
    public final String expected;
    public final String actual;
    public final String message;

    public Result(
        boolean ok,
        String status,
        String algorithm,
        String expected,
        String actual,
        String message) {
      this.ok = ok;
      this.status = status;
      this.algorithm = algorithm;
      this.expected = expected;
      this.actual = actual;
      this.message = message;
    }

    @Override
    public String toString() {
      return "Result{ok="
          + ok
          + ", status="
          + status
          + ", algorithm="
          + algorithm
          + ", expected="
          + expected
          + ", actual="
          + actual
          + ", message="
          + message
          + "}";
    }
  }

  public static Result checkDocumentText(String text) {
    return checkDocumentText(text, SEAL_FIELD);
  }

  public static Result checkDocumentText(String text, String field) {
    ParsedDoc doc = parseDocument(text);
    if (!doc.hadFrontMatter) {
      Digest actual = computeDigest(doc);
      return new Result(false, "missing_seal", "blake3", null, actual.qualified, "missing hashseal field");
    }
    String sealRaw = extractReservedField(doc.fmLines, field);
    if (sealRaw == null) {
      Digest actual = computeDigest(doc);
      return new Result(false, "missing_seal", "blake3", null, actual.qualified, "missing hashseal field");
    }
    Digest expected = parseDigest(sealRaw);
    if (expected == null) {
      return new Result(
          false, "invalid_format", null, null, null, "invalid digest format: " + sealRaw);
    }
    if (!"blake3".equals(expected.algorithm)) {
      return new Result(
          false,
          "invalid_format",
          expected.algorithm,
          expected.qualified,
          null,
          "unsupported algorithm: " + expected.algorithm);
    }
    Digest actual = computeDigest(doc);
    if (!actual.hex.equals(expected.hex) || !actual.algorithm.equals(expected.algorithm)) {
      return new Result(
          false, "mismatch", expected.algorithm, expected.qualified, actual.qualified, null);
    }
    return new Result(true, "valid", actual.algorithm, expected.qualified, actual.qualified, null);
  }

  public static Digest blake3Digest(byte[] data) {
    String hex = Blake3.hashHex(data);
    return new Digest("blake3", hex, "blake3:" + hex);
  }

  public static Digest blake3Digest(String utf8) {
    return blake3Digest(utf8.getBytes(StandardCharsets.UTF_8));
  }

  public static final class Digest {
    public final String algorithm;
    public final String hex;
    public final String qualified;

    public Digest(String algorithm, String hex, String qualified) {
      this.algorithm = algorithm;
      this.hex = hex;
      this.qualified = qualified;
    }
  }

  static final class ParsedDoc {
    final List<String> fmLines;
    final boolean hadFrontMatter;
    final String bodyRaw;

    ParsedDoc(List<String> fmLines, boolean hadFrontMatter, String bodyRaw) {
      this.fmLines = fmLines;
      this.hadFrontMatter = hadFrontMatter;
      this.bodyRaw = bodyRaw;
    }
  }

  static String stripBom(String s) {
    if (s != null && !s.isEmpty() && s.charAt(0) == '\uFEFF') {
      return s.substring(1);
    }
    return s;
  }

  static String normalizeLf(String s) {
    return s.replace("\r\n", "\n").replace("\r", "\n");
  }

  static ParsedDoc parseDocument(String text) {
    text = stripBom(text);
    if (text.startsWith("---\n") || text.startsWith("---\r\n")) {
      String afterOpen = text.startsWith("---\r\n") ? text.substring(5) : text.substring(4);
      String search = afterOpen;
      int offset = 0;
      while (true) {
        int idx = search.indexOf("\n---");
        if (idx < 0) {
          break;
        }
        String after = search.substring(idx + 1);
        String rest = after.substring(3);
        boolean closed =
            rest.isEmpty()
                || rest.startsWith("\n")
                || rest.startsWith("\r\n")
                || rest.startsWith("\r");
        if (closed) {
          String fmBlock = afterOpen.substring(0, offset + idx);
          String body = afterOpen.substring(idx + 1 + 3);
          if (body.startsWith("\r\n")) {
            body = body.substring(2);
          } else if (body.startsWith("\n")) {
            body = body.substring(1);
          } else if (body.startsWith("\r")) {
            body = body.substring(1);
          }
          String[] parts = normalizeLf(fmBlock).split("\n", -1);
          List<String> fmLines = new ArrayList<>(parts.length);
          for (String p : parts) {
            fmLines.add(p);
          }
          return new ParsedDoc(fmLines, true, body);
        }
        offset += idx + 1;
        search = search.substring(idx + 1);
      }
    }
    return new ParsedDoc(new ArrayList<>(), false, text);
  }

  static boolean isReservedKey(String key) {
    return SEAL_FIELD.equals(key) || SIG_FIELD.equals(key) || KEY_ID_FIELD.equals(key);
  }

  static void forEachFmEntry(List<String> lines, BiConsumer<String, String> f) {
    int i = 0;
    int n = lines.size();
    while (i < n) {
      String line = lines.get(i);
      String trimmed = line.trim();
      if (trimmed.isEmpty() || trimmed.startsWith("#")) {
        i++;
        continue;
      }
      if (line.startsWith(" ") || line.startsWith("\t")) {
        i++;
        continue;
      }
      int colon = trimmed.indexOf(':');
      if (colon >= 0) {
        String key = trimmed.substring(0, colon).trim();
        String rest = trimmed.substring(colon + 1).trim();
        if (isReservedKey(key)) {
          i++;
          while (i < n) {
            String L = lines.get(i);
            if (L.startsWith(" ") || L.startsWith("\t")) {
              i++;
              continue;
            }
            if (L.trim().isEmpty()) {
              if (i + 1 < n
                  && (lines.get(i + 1).startsWith(" ") || lines.get(i + 1).startsWith("\t"))) {
                i++;
                continue;
              }
              break;
            }
            break;
          }
          continue;
        }
        if ("|".equals(rest) || ">".equals(rest) || "|-".equals(rest) || ">-".equals(rest)) {
          StringBuilder val = new StringBuilder();
          i++;
          while (i < n
              && (lines.get(i).startsWith(" ") || lines.get(i).startsWith("\t"))) {
            if (val.length() > 0) {
              val.append('\n');
            }
            val.append(lines.get(i).replaceFirst("^[ \\t]+", ""));
            i++;
          }
          f.accept(key, val.toString());
          continue;
        }
        String val = rest;
        if (val.startsWith("\"") && val.endsWith("\"") && val.length() >= 2) {
          val = val.substring(1, val.length() - 1);
        }
        f.accept(key, val);
      }
      i++;
    }
  }

  static Map<String, String> fmMap(List<String> lines) {
    Map<String, String> map = new LinkedHashMap<>();
    forEachFmEntry(lines, map::put);
    return map;
  }

  static String extractReservedField(List<String> lines, String field) {
    int i = 0;
    int n = lines.size();
    while (i < n) {
      String trimmed = lines.get(i).trim();
      int colon = trimmed.indexOf(':');
      if (colon >= 0) {
        String k = trimmed.substring(0, colon).trim();
        if (k.equals(field)) {
          String rest = trimmed.substring(colon + 1).trim();
          if ("|".equals(rest) || ">".equals(rest) || "|-".equals(rest) || ">-".equals(rest)) {
            StringBuilder val = new StringBuilder();
            i++;
            while (i < n) {
              String L = lines.get(i);
              boolean empty = L.trim().isEmpty();
              boolean indented = L.startsWith(" ") || L.startsWith("\t");
              if (indented
                  || (empty
                      && i + 1 < n
                      && (lines.get(i + 1).startsWith(" ")
                          || lines.get(i + 1).startsWith("\t")))) {
                if (empty) {
                  val.append('\n');
                  i++;
                  continue;
                }
                if (val.length() > 0) {
                  val.append('\n');
                }
                val.append(L.replaceFirst("^[ \\t]+", ""));
                i++;
                continue;
              }
              break;
            }
            return val.toString();
          }
          if (rest.startsWith("\"") && rest.endsWith("\"") && rest.length() >= 2) {
            rest = rest.substring(1, rest.length() - 1);
          }
          return rest;
        }
      }
      i++;
    }
    return null;
  }

  static String canonicalFmString(Map<String, String> map) {
    TreeMap<String, String> sorted = new TreeMap<>(map);
    StringBuilder s = new StringBuilder();
    for (Map.Entry<String, String> e : sorted.entrySet()) {
      String k = e.getKey();
      String v = e.getValue();
      s.append(k).append(": ");
      if (v.isEmpty() || v.indexOf(':') >= 0 || v.indexOf('#') >= 0 || v.indexOf(' ') >= 0) {
        s.append('"').append(v.replace("\"", "\\\"")).append('"');
      } else {
        s.append(v);
      }
      s.append('\n');
    }
    return s.toString();
  }

  static byte[] hashPayload(ParsedDoc doc) {
    String bodyLf = normalizeLf(doc.bodyRaw);
    Map<String, String> map = fmMap(doc.fmLines);
    if (map.isEmpty()) {
      return bodyLf.getBytes(StandardCharsets.UTF_8);
    }
    String payload = canonicalFmString(map) + "\n" + bodyLf;
    return payload.getBytes(StandardCharsets.UTF_8);
  }

  static Digest computeDigest(ParsedDoc doc) {
    return blake3Digest(hashPayload(doc));
  }

  static Digest parseDigest(String raw) {
    String s = raw.trim();
    if (s.startsWith("\"") && s.endsWith("\"") && s.length() >= 2) {
      s = s.substring(1, s.length() - 1);
    }
    int idx = s.indexOf(':');
    if (idx < 0) {
      return null;
    }
    String algorithm = s.substring(0, idx).toLowerCase();
    String hex = s.substring(idx + 1).trim().toLowerCase();
    if (hex.isEmpty()) {
      return null;
    }
    for (int i = 0; i < hex.length(); i++) {
      char c = hex.charAt(i);
      if ((c < '0' || c > '9') && (c < 'a' || c > 'f')) {
        return null;
      }
    }
    return new Digest(algorithm, hex, algorithm + ":" + hex);
  }
}
