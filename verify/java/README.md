---
hashseal: "blake3:385376d28978c7a5155558a5a6b9048de5fe3d8b98f206d34e98fe1f7c92b21c"
---
# hashseal-verify (Java)

Zero-dependency HashSeal instruct document verifier (pure Java).

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- JDK 11+ (tested on 17/21/25)
- **No Maven runtime dependencies** for the verify library

## Build & test

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

Optional jar (still no external deps):

```bash
jar --create --file hashseal-verify.jar -C out ai
```

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

## Vendor

`Blake3.java` is a pure-Java port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual). No third-party jars.

```text
Copyright (c) 2026 MonkeyKing.dev
```
