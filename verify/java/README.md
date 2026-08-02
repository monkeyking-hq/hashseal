---
hashseal: "blake3:385376d28978c7a5155558a5a6b9048de5fe3d8b98f206d34e98fe1f7c92b21c"
---
# hashseal-verify (Java)

Zero-dependency HashSeal instruct document + tree verifier (pure Java).

**Signed, Sealed, Delivered - I'm Yours.**

**Maven coordinates:** `ai.hashseal:hashseal-verify` (parent reactor: [`java/`](../../java/))

## Requirements

- JDK 11+ (tested on 17/21/25)
- **No Maven runtime dependencies** for the verify library

## Build & test

### Maven (preferred)

From monorepo root:

```bash
mvn -f verify/java/pom.xml clean test
# or whole Java reactor:
mvn -f java/pom.xml clean install
```

Installs `ai.hashseal:hashseal-verify` to the local repo. Vector runners run in the `test` phase.

### Manual javac (no Maven)

From `verify/java/`:

```bash
# Windows (PowerShell)
mkdir out -Force
javac -d out src/ai/hashseal/verify/*.java test/RunVectors.java test/RunTreeVectors.java
java -cp out RunVectors
java -cp out RunTreeVectors
```

```bash
# Unix
mkdir -p out
javac -d out src/ai/hashseal/verify/*.java test/RunVectors.java test/RunTreeVectors.java
java -cp out RunVectors
java -cp out RunTreeVectors
```

Uses frozen vectors at `../vectors/instruct-v1.json` and `../vectors/tree-v1.json`.

## Consumer dependency

After a Central publish (or local `mvn install`):

```xml
<dependency>
  <groupId>ai.hashseal</groupId>
  <artifactId>hashseal-verify</artifactId>
  <version>0.1.0-SNAPSHOT</version> <!-- use a release version once published -->
</dependency>
```

Maven Central deploy is wired on the parent (`-Pcentral`, server id `hashseal-central`). Versioning/tags: `mvn -f java/pom.xml release:prepare release:perform`. See [`java/README.md`](../../java/README.md). Do not publish from agent builds unless asked.

## API

```java
import ai.hashseal.verify.Check;
import ai.hashseal.verify.Tree;

Check.Result r = Check.checkDocumentText(markdownText);
// r.ok, r.status ("valid"|"mismatch"|"missing_seal"|"invalid_format")
// r.algorithm, r.expected, r.actual, r.message

Tree.FileHash h = Tree.hashTreeFileContent("src/a.txt", "hello\n");
Tree.VerifyResult tv = Tree.verifyTreeInMemory(
    Map.of("src/a.txt", "hello\n"),
    List.of(new Tree.LedgerEntry("src/a.txt", h.digest, h.size)));
// tv.ok, tv.checked, tv.findings — every non-OK path listed
```

Digest check only (FULL canonical mode). GPG signature verification is not performed here.
**This library does not seal** — use the `hashseal` CLI (or the Maven plugin) for seal.

## Vendor

`Blake3.java` is a pure-Java port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual). No third-party jars.

```text
Copyright (c) 2026 MonkeyKing.dev
```
