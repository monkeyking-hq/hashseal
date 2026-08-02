import ai.hashseal.verify.Tree;

import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * Run official tree-v1 vectors. Zero Maven deps — minimal JSON scrape.
 *
 * Usage (from verify/java):
 *   javac -d out src/ai/hashseal/verify/*.java test/RunTreeVectors.java
 *   java -cp out RunTreeVectors
 */
public final class RunTreeVectors {
  public static void main(String[] args) throws Exception {
    Path vectors =
        Path.of("..")
            .resolve("vectors")
            .resolve("tree-v1.json")
            .toAbsolutePath()
            .normalize();
    if (!Files.isRegularFile(vectors)) {
      Path alt = Path.of("verify/vectors/tree-v1.json").toAbsolutePath().normalize();
      if (Files.isRegularFile(alt)) {
        vectors = alt;
      } else {
        System.err.println("vectors not found: " + vectors);
        System.exit(1);
      }
    }
    String json = Files.readString(vectors, StandardCharsets.UTF_8);
    if (!json.contains("\"tree-v1\"")) {
      System.err.println("unexpected vectors file");
      System.exit(1);
    }

    boolean lfText = true;
    Boolean lfField = boolField(json, "line_endings_lf_text");
    if (lfField != null) {
      lfText = lfField;
    }
    List<String> textExtList = stringArrayField(json, "text_extensions");
    Set<String> textExts =
        textExtList != null ? Tree.textExtensionsFromList(textExtList) : null;

    List<Case> cases = parseCases(json);
    int passed = 0;
    int failed = 0;
    for (Case c : cases) {
      try {
        if ("raw_file_digest".equals(c.kind)) {
          Tree.FileHash r =
              Tree.hashTreeFileContent(c.path, c.content, lfText, textExts);
          assertEq(r.digest, c.expectDigest, c.id + " digest");
          assertEq(Integer.valueOf(r.size), Integer.valueOf(c.expectSize), c.id + " size");
        } else if ("verify_tree".equals(c.kind)) {
          Map<String, String> files = c.files != null ? c.files : new HashMap<>();
          List<Tree.LedgerEntry> ledger = new ArrayList<>();
          if (c.ledger != null) {
            for (LedgerRow row : c.ledger) {
              ledger.add(new Tree.LedgerEntry(row.path, row.digest, row.size));
            }
          }
          Tree.VerifyResult r = Tree.verifyTreeInMemory(files, ledger, lfText, textExts);
          assertEq(Boolean.valueOf(r.ok), Boolean.valueOf(c.expectOk), c.id + " ok");
          assertEq(
              Integer.valueOf(r.checked), Integer.valueOf(c.expectChecked), c.id + " checked");
          List<FindingExpect> want = c.expectFindings != null ? c.expectFindings : List.of();
          assertEq(
              Integer.valueOf(r.findings.size()),
              Integer.valueOf(want.size()),
              c.id + " findings.length");
          for (int i = 0; i < want.size(); i++) {
            Tree.Finding g = r.findings.get(i);
            FindingExpect w = want.get(i);
            assertEq(g.path, w.path, c.id + " finding[" + i + "].path");
            assertEq(g.status, w.status, c.id + " finding[" + i + "].status");
            assertEq(g.expected, w.expected, c.id + " finding[" + i + "].expected");
            assertEq(g.actual, w.actual, c.id + " finding[" + i + "].actual");
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
    String path;
    String content;
    Map<String, String> files;
    List<LedgerRow> ledger;
    boolean expectOk;
    int expectChecked;
    String expectDigest;
    int expectSize;
    List<FindingExpect> expectFindings;
  }

  static final class LedgerRow {
    String path;
    String digest;
    int size;
  }

  static final class FindingExpect {
    String path;
    String status;
    String expected;
    String actual;
  }

  static List<Case> parseCases(String json) {
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
      c.path = stringField(obj, "path");
      c.content = stringField(obj, "content");
      c.files = stringMapField(obj, "files");
      c.ledger = parseLedger(objectArrayField(obj, "ledger_entries"));
      String expectBlock = objectField(obj, "expect");
      if (expectBlock != null) {
        Boolean ok = boolField(expectBlock, "ok");
        c.expectOk = ok != null && ok;
        Integer checked = intField(expectBlock, "checked");
        c.expectChecked = checked != null ? checked : 0;
        c.expectDigest = stringField(expectBlock, "digest");
        Integer size = intField(expectBlock, "size");
        c.expectSize = size != null ? size : 0;
        c.expectFindings = parseFindings(objectArrayField(expectBlock, "findings"));
      }
      out.add(c);
    }
    return out;
  }

  static List<LedgerRow> parseLedger(List<String> objs) {
    if (objs == null) {
      return new ArrayList<>();
    }
    List<LedgerRow> out = new ArrayList<>();
    for (String o : objs) {
      LedgerRow row = new LedgerRow();
      row.path = stringField(o, "path");
      row.digest = stringField(o, "digest");
      Integer size = intField(o, "size");
      row.size = size != null ? size : 0;
      out.add(row);
    }
    return out;
  }

  static List<FindingExpect> parseFindings(List<String> objs) {
    if (objs == null) {
      return new ArrayList<>();
    }
    List<FindingExpect> out = new ArrayList<>();
    for (String o : objs) {
      FindingExpect f = new FindingExpect();
      f.path = stringField(o, "path");
      f.status = stringField(o, "status");
      f.expected = nullableStringField(o, "expected");
      f.actual = nullableStringField(o, "actual");
      out.add(f);
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

  /** String field that may be JSON null. */
  static String nullableStringField(String obj, String name) {
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
    if (i >= obj.length()) {
      return null;
    }
    if (obj.startsWith("null", i)) {
      return null;
    }
    if (obj.charAt(i) != '"') {
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

  static Map<String, String> stringMapField(String obj, String name) {
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
    String body = obj.substring(i + 1, end);
    Map<String, String> m = new HashMap<>();
    int pos = 0;
    while (pos < body.length()) {
      while (pos < body.length() && (Character.isWhitespace(body.charAt(pos)) || body.charAt(pos) == ',')) {
        pos++;
      }
      if (pos >= body.length()) {
        break;
      }
      if (body.charAt(pos) != '"') {
        break;
      }
      String k = parseJsonString(body, pos);
      int afterKey = skipJsonString(body, pos);
      int c = body.indexOf(':', afterKey);
      if (c < 0) {
        break;
      }
      int v = c + 1;
      while (v < body.length() && Character.isWhitespace(body.charAt(v))) {
        v++;
      }
      if (v >= body.length() || body.charAt(v) != '"') {
        break;
      }
      String val = parseJsonString(body, v);
      m.put(k, val);
      pos = skipJsonString(body, v);
    }
    return m;
  }

  static List<String> objectArrayField(String obj, String name) {
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
    if (i >= obj.length() || obj.charAt(i) != '[') {
      return null;
    }
    int end = findMatching(obj, i, '[', ']');
    return splitTopObjects(obj.substring(i + 1, end));
  }

  static List<String> stringArrayField(String json, String name) {
    String key = "\"" + name + "\"";
    int idx = indexOfKey(json, key);
    if (idx < 0) {
      return null;
    }
    int colon = json.indexOf(':', idx + key.length());
    int i = colon + 1;
    while (i < json.length() && Character.isWhitespace(json.charAt(i))) {
      i++;
    }
    if (i >= json.length() || json.charAt(i) != '[') {
      return null;
    }
    int end = findMatching(json, i, '[', ']');
    String inner = json.substring(i + 1, end);
    List<String> out = new ArrayList<>();
    int pos = 0;
    while (pos < inner.length()) {
      while (pos < inner.length()
          && (Character.isWhitespace(inner.charAt(pos)) || inner.charAt(pos) == ',')) {
        pos++;
      }
      if (pos >= inner.length()) {
        break;
      }
      if (inner.charAt(pos) != '"') {
        break;
      }
      out.add(parseJsonString(inner, pos));
      pos = skipJsonString(inner, pos);
    }
    return out;
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

  static Integer intField(String obj, String name) {
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
    int start = i;
    if (i < obj.length() && obj.charAt(i) == '-') {
      i++;
    }
    while (i < obj.length() && Character.isDigit(obj.charAt(i))) {
      i++;
    }
    if (start == i || (i == start + 1 && obj.charAt(start) == '-')) {
      return null;
    }
    return Integer.parseInt(obj.substring(start, i));
  }

  static int indexOfKey(String obj, String key) {
    int from = 0;
    while (true) {
      int idx = obj.indexOf(key, from);
      if (idx < 0) {
        return -1;
      }
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

  static int skipJsonString(String s, int quoteIdx) {
    boolean esc = false;
    for (int i = quoteIdx + 1; i < s.length(); i++) {
      char ch = s.charAt(i);
      if (esc) {
        if (ch == 'u') {
          i += 4;
        }
        esc = false;
        continue;
      }
      if (ch == '\\') {
        esc = true;
        continue;
      }
      if (ch == '"') {
        return i + 1;
      }
    }
    throw new IllegalArgumentException("unterminated string");
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
