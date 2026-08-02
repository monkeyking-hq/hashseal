---
hashseal: "blake3:96d45ca14a2e7fddbd714df04226129ea563a5eae5163610ce13135fd0954980"
---
# hashseal-gradle-plugin

Thin **Gradle** plugin that shells out to the **`hashseal` CLI**.

**Signed, Sealed, Delivered - I'm Yours.**

## Tasks

| Task | CLI |
|------|-----|
| `hashsealSealInstruct` | `hashseal seal --instruct --root …` |
| `hashsealCheck` | `hashseal check --root …` |
| `hashsealVerify` | `hashseal verify --root …` |
| `hashsealSealTree` | `hashseal seal --tree --release --root …` |

## Apply (composite / includeBuild)

```kotlin
// settings.gradle.kts
includeBuild("../path/to/hashseal/plugins/gradle")

// build.gradle.kts
plugins {
  id("ai.hashseal")
}

hashseal {
  // bin.set(System.getenv("HASHSEAL_BIN") ?: "")
  // root.set(layout.projectDirectory.asFile.absolutePath)
  // sign.set(false)
}

tasks.named("check") {
  dependsOn("hashsealCheck")
}
```

## Binary resolution

1. `hashseal.bin` extension property  
2. `HASHSEAL_BIN` env  
3. `hashseal` on **PATH**

See [docs/packaging.md](../../docs/packaging.md).

## Build this plugin

```bash
cd plugins/gradle
./gradlew jar   # or gradle jar
```

Requires JDK 11+ and Gradle (wrapper optional).

```text
Copyright (c) 2026 MonkeyKing.dev
```
