---
hashseal: "blake3:f5ac5bbb91ec451a7cb49849374215ad402e0d0368297620af3def1d41a6f89c"
---
# hashseal-maven-plugin

Thin **Maven** Mojo(s) that shell out to the **`hashseal` CLI**.

**Signed, Sealed, Delivered - I'm Yours.**

## PATH requirement

The plugin does **not** ship native binaries. Provide the CLI via:

| Priority | Source |
|----------|--------|
| 1 | `-Dhashseal.bin=/path/to/hashseal` |
| 2 | env `HASHSEAL_BIN` |
| 3 | bare `hashseal` on **PATH** |

Build CLI from this monorepo:

```bash
cargo build -p hashseal --release
```

## Goals

| Goal | CLI | Default phase |
|------|-----|----------------|
| `hashseal:seal` | `hashseal seal` | `process-resources` |
| `hashseal:check` | `hashseal check` | `verify` |
| `hashseal:verify` | `hashseal verify` | `verify` |

### Parameters (common)

| Property | Description |
|----------|-------------|
| `hashseal.bin` | Absolute path to CLI |
| `hashseal.root` | `--root` (default: `${project.basedir}`) |
| `hashseal.skip` | Skip goal |
| `hashseal.instruct` | seal: `--instruct` (default true) |
| `hashseal.tree` | seal: `--tree` |
| `hashseal.release` | seal: `--release` |
| `hashseal.sign` | seal: `--sign` |
| `hashseal.requireSignature` | check: `--require-signature` |

## Local install

From monorepo root (preferred — uses shared parent):

```bash
mvn -f java/pom.xml clean install
```

Or only this plugin:

```bash
cd plugins/maven
mvn -q install -DskipTests
```

Consumer (after local install or Central publish):

```xml
<plugin>
  <groupId>ai.hashseal</groupId>
  <artifactId>hashseal-maven-plugin</artifactId>
  <version>0.1.0-SNAPSHOT</version>
  <executions>
    <execution>
      <id>check</id>
      <goals><goal>check</goal></goals>
    </execution>
  </executions>
  <configuration>
    <!-- optional: <bin>${env.HASHSEAL_BIN}</bin> -->
  </configuration>
</plugin>
```

Or one-shot:

```bash
mvn ai.hashseal:hashseal-maven-plugin:0.1.0-SNAPSHOT:check -Dhashseal.root=../../fixtures/mvp-demo
```

## Maven Central

Publishing is configured on the Java parent (`java/pom.xml`) with Portal server id **`hashseal-central`**.

```bash
# Manual version
mvn -f java/pom.xml clean deploy -Pcentral

# Version bump + tag + deploy (clean tree)
mvn -f java/pom.xml release:prepare release:perform
```

See [`java/README.md`](../../java/README.md). Operator-only; agent builds do not publish unless requested.

## Notes

- Wire-up for Central is present (`-Pcentral`); do **not** publish from agent builds unless requested.
- Compile-time deps are Maven API only (`provided` scope).
- In-app check library (no CLI): `ai.hashseal:hashseal-verify` under `verify/java`.
- Failures surface CLI exit codes; the CLI prints every non-OK path.

```text
Copyright (c) 2026 MonkeyKing.dev
```
