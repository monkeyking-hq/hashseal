import ai.hashseal.verify.Check;
import ai.hashseal.verify.Blake3;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Run official instruct-v1 vectors. Zero Maven deps — uses minimal JSON scrape.
 *
 * Usage (from verify/java):
 *   javac -d out src/ai/hashseal/verify/*.java test/RunVectors.java
 *   java -cp out RunVectors
 */
public final class RunVectors {
  public static void main(String[] args) throws Exception {
    Path vectors =
        Path.of("..")
            .resolve("vectors")
            .resolve("instruct-v1.json")
            .toAbsolutePath()
            .normalize();
    if (!Files.isRegularFile(vectors)) {
      // allow running from repo root
      Path alt = Path.of("verify/vectors/instruct-v1.json").toAbsolutePath().normalize();
      if (Files.isRegularFile(alt)) {
        vectors = alt;
      } else {
        System.err.println("vectors not found: " + vectors);
        System.exit(1);
      }
    }
    String json = Files.readString(vectors, StandardCharsets.UTF_8);
    if (!json.contains("\"instruct-v1\"")) {
      System.err.println("unexpected vectors file");
      System.exit(1);
    }

    List<Case> cases = parseCases(json);
    int passed = 0;
    int failed = 0;
    for (Case c : cases) {
      try {
        if ("raw_digest".equals(c.kind)) {
          String actual = Check.blake3Digest(c.bytesUtf8).qualified;
          assertEq(actual, c.expectDigest, c.id + " digest");
        } else if ("check".equals(c.kind)) {
          Check.Result r = Check.checkDocumentText(c.text);
          assertEq(Boolean.valueOf(r.ok), Boolean.valueOf(c.expectOk), c.id + " ok");
          assertEq(r.status, c.expectStatus, c.id + " status");
          if (c.expectDigest != null) {
            assertEq(r.actual, c.expectDigest, c.id + " actual digest");
            if (r.ok) {
              assertEq(r.expected, c.expectDigest, c.id + " expected digest");
            }
          }
          if (c.expectExpected != null) {
            assertEq(r.expected, c.expectExpected, c.id + " expected");
          }
          if (c.expectActual != null) {
            assertEq(r.actual, c.expectActual, c.id + " actual");
          }
        } else {
          throw new RuntimeException("unknown kind " + c.kind);
        }
        passed++;
        System.out.println("ok  " + c.id);
      } catch (Throwable e) {
        failed++;
        System.err.println("FAIL " + c.id + ": " + e.getMessage());
      }
    }
    System.out.println();
    System.out.println(passed + " passed, " + failed + " failed");
    System.exit(failed == 0 ? 0 : 1);
  }

  private static void assertEq(Object a, Object b, String label) {
    if (a == null ? b != null : !a.equals(b)) {
      throw new AssertionError(label + ": got " + a + " want " + b);
    }
  }

  static final class Case {
    String id;
    String kind;
    String text;
    String bytesUtf8;
    boolean expectOk;
    String expectStatus;
    String expectDigest;
    String expectExpected;
    String expectActual;
  }

  /**
   * Minimal JSON parser for our frozen vector shape only.
   * Avoids requiring org.json / Jackson.
   */
  static List<Case> parseCases(String json) {
    // Find "cases": [ ... ]
    int casesIdx = json.indexOf("\"cases\"");
    if (casesIdx < 0) {
      throw new IllegalArgumentException("no cases");
    }
    int arrStart = json.indexOf('[', casesIdx);
    int arrEnd = findMatching(json, arrStart, '[', ']');
    String arr = json.substring(arrStart + 1, arrEnd);
    List<String> objects = splitTopObjects(arr);
    List<Case> out = new ArrayList<>();
    for (String obj : objects) {
      Case c = new Case();
      c.id = stringField(obj, "id");
      c.kind = stringField(obj, "kind");
      c.text = stringField(obj, "text");
      c.bytesUtf8 = stringField(obj, "bytes_utf8");
      String expectBlock = objectField(obj, "expect");
      if (expectBlock != null) {
        Boolean ok = boolField(expectBlock, "ok");
        c.expectOk = ok != null && ok;
        c.expectStatus = stringField(expectBlock, "status");
        c.expectDigest = stringField(expectBlock, "digest");
        c.expectExpected = stringField(expectBlock, "expected");
        c.expectActual = stringField(expectBlock, "actual");
      }
      out.add(c);
    }
    return out;
  }

  static int findMatching(String s, int openIdx, char open, char close) {
    int depth = 0;
    boolean inStr = false;
    boolean esc = false;
    for (int i = openIdx; i < s.length(); i++) {
      char ch = s.charAt(i);
      if (inStr) {
        if (esc) {
          esc = false;
        } else if (ch == '\\') {
          esc = true;
        } else if (ch == '"') {
          inStr = false;
        }
        continue;
      }
      if (ch == '"') {
        inStr = true;
        continue;
      }
      if (ch == open) {
        depth++;
      } else if (ch == close) {
        depth--;
        if (depth == 0) {
          return i;
        }
      }
    }
    throw new IllegalArgumentException("unbalanced " + open + close);
  }

  static List<String> splitTopObjects(String arrInner) {
    List<String> out = new ArrayList<>();
    int i = 0;
    while (i < arrInner.length()) {
      while (i < arrInner.length() && Character.isWhitespace(arrInner.charAt(i))) {
        i++;
      }
      if (i >= arrInner.length()) {
        break;
      }
      if (arrInner.charAt(i) == ',') {
        i++;
        continue;
      }
      if (arrInner.charAt(i) != '{') {
        i++;
        continue;
      }
      int end = findMatching(arrInner, i, '{', '}');
      out.add(arrInner.substring(i, end + 1));
      i = end + 1;
    }
    return out;
  }

  static String stringField(String obj, String name) {
    String key = "\"" + name + "\"";
    int idx = indexOfKey(obj, key);
    if (idx < 0) {
      return null;
    }
    int colon = obj.indexOf(':', idx + key.length());
    int i = colon + 1;
    while (i < obj.length() && Character.isWhitespace(obj.charAt(i))) {
      i++;
    }
    if (i >= obj.length() || obj.charAt(i) != '"') {
      return null;
    }
    return parseJsonString(obj, i);
  }

  static String objectField(String obj, String name) {
    String key = "\"" + name + "\"";
    int idx = indexOfKey(obj, key);
    if (idx < 0) {
      return null;
    }
    int colon = obj.indexOf(':', idx + key.length());
    int i = colon + 1;
    while (i < obj.length() && Character.isWhitespace(obj.charAt(i))) {
      i++;
    }
    if (i >= obj.length() || obj.charAt(i) != '{') {
      return null;
    }
    int end = findMatching(obj, i, '{', '}');
    return obj.substring(i, end + 1);
  }

  static Boolean boolField(String obj, String name) {
    String key = "\"" + name + "\"";
    int idx = indexOfKey(obj, key);
    if (idx < 0) {
      return null;
    }
    int colon = obj.indexOf(':', idx + key.length());
    int i = colon + 1;
    while (i < obj.length() && Character.isWhitespace(obj.charAt(i))) {
      i++;
    }
    if (obj.startsWith("true", i)) {
      return true;
    }
    if (obj.startsWith("false", i)) {
      return false;
    }
    return null;
  }

  /** Prefer top-level key match (simple: first occurrence after a structural boundary). */
  static int indexOfKey(String obj, String key) {
    int from = 0;
    while (true) {
      int idx = obj.indexOf(key, from);
      if (idx < 0) {
        return -1;
      }
      // ensure not mid-string of a value: previous non-space should be { or ,
      int j = idx - 1;
      while (j >= 0 && Character.isWhitespace(obj.charAt(j))) {
        j--;
      }
      if (j < 0 || obj.charAt(j) == '{' || obj.charAt(j) == ',') {
        return idx;
      }
      from = idx + 1;
    }
  }

  static String parseJsonString(String s, int quoteIdx) {
    StringBuilder sb = new StringBuilder();
    boolean esc = false;
    for (int i = quoteIdx + 1; i < s.length(); i++) {
      char ch = s.charAt(i);
      if (esc) {
        switch (ch) {
          case '"':
          case '\\':
          case '/':
            sb.append(ch);
            break;
          case 'b':
            sb.append('\b');
            break;
          case 'f':
            sb.append('\f');
            break;
          case 'n':
            sb.append('\n');
            break;
          case 'r':
            sb.append('\r');
            break;
          case 't':
            sb.append('\t');
            break;
          case 'u':
            int code = Integer.parseInt(s.substring(i + 1, i + 5), 16);
            sb.append((char) code);
            i += 4;
            break;
          default:
            sb.append(ch);
            break;
        }
        esc = false;
        continue;
      }
      if (ch == '\\') {
        esc = true;
        continue;
      }
      if (ch == '"') {
        return sb.toString();
      }
      sb.append(ch);
    }
    throw new IllegalArgumentException("unterminated string");
  }
}
