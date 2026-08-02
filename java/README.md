# HashSeal Java (Maven reactor)

Parent POM and reactor for Java artifacts published under the **`ai.hashseal`** Maven Central namespace.

**Signed, Sealed, Delivered - I'm Yours.**

## Modules

| Artifact | Path | Role |
|----------|------|------|
| `ai.hashseal:hashseal-java-parent` | `java/` | Parent / shared Central + release config |
| `ai.hashseal:hashseal-verify` | `verify/java/` | Zero-dep **check** library (instruct + tree) |
| `ai.hashseal:hashseal-maven-plugin` | `plugins/maven/` | Thin Mojo(s) → `hashseal` CLI |

There is **no** Java seal library yet; seal stays on the CLI. Apps embed **`hashseal-verify`** for in-process check.

## Local build / install

From the monorepo root:

```bash
mvn -f java/pom.xml clean install
```

Or only the verify library:

```bash
mvn -f verify/java/pom.xml clean install
```

## Maven Central (Portal)

### Prerequisites

1. Namespace **`ai.hashseal`** registered on [Central Portal](https://central.sonatype.com/).
2. `~/.m2/settings.xml` server id **`hashseal-central`** with a [user token](https://central.sonatype.org/publish/generate-portal-token/):

```xml
<servers>
  <server>
    <id>hashseal-central</id>
    <username><!-- token username --></username>
    <password><!-- token password --></password>
  </server>
</servers>
```

3. GPG key available to `maven-gpg-plugin` (passwordless agent is fine); **public key published** to a keyserver.
4. Git push access matching `scm.developerConnection` (SSH to `github.com/monkeyking-hq/hashseal`).
5. **Clean git working tree** before `release:prepare` (commit or stash unrelated changes).

### One-shot deploy (manual version)

```bash
# After editing version to a non-SNAPSHOT release in all three POMs:
mvn -f java/pom.xml clean deploy -Pcentral

# Auto-publish after Portal validation (optional)
mvn -f java/pom.xml clean deploy -Pcentral -Dhashseal.central.autoPublish=true
```

### Release plugin (versioning + tag + deploy)

[`maven-release-plugin`](https://maven.apache.org/maven-release/maven-release-plugin/) is configured on the parent:

| Setting | Value |
|---------|--------|
| Tag format | `hashseal-java-@{project.version}` (e.g. `hashseal-java-0.1.0`) |
| Submodules | `autoVersionSubmodules=true` (parent + children stay aligned) |
| Profiles | `releaseProfiles=central` (sources, javadoc, GPG, Portal plugin) |
| Deploy goal | `deploy` |
| Checkout | `localCheckout=true` (monorepo-friendly) |

```bash
# Dry-run (no commits / tags / push)
mvn -f java/pom.xml release:prepare -DdryRun=true

# Real release: bump off SNAPSHOT → tag → next SNAPSHOT → push → deploy tag with -Pcentral
mvn -f java/pom.xml release:prepare release:perform

# If prepare fails mid-way:
#   mvn -f java/pom.xml release:rollback
# or release:clean after fixing
```

`release:prepare` prompts (or accepts `-DreleaseVersion=0.1.0 -DdevelopmentVersion=0.1.1-SNAPSHOT -Dtag=hashseal-java-0.1.0`) for:

1. **release version** (no `-SNAPSHOT`)
2. **SCM tag** (default from `tagNameFormat`)
3. **next development version** (`…-SNAPSHOT`)

`release:perform` builds the tag and runs **`deploy` with `-Pcentral`**. With default `autoPublish=false`, finish the deployment in the [Portal UI](https://central.sonatype.com/publishing/deployments) after validation.

Server id is fixed as `hashseal.central.serverId=hashseal-central`.

## Consumer coordinates (after publish)

```xml
<dependency>
  <groupId>ai.hashseal</groupId>
  <artifactId>hashseal-verify</artifactId>
  <version>0.1.0</version>
</dependency>
```

```xml
<plugin>
  <groupId>ai.hashseal</groupId>
  <artifactId>hashseal-maven-plugin</artifactId>
  <version>0.1.0</version>
</plugin>
```

Until a non-SNAPSHOT is published, use `mvn install` locally or a SNAPSHOT repo if you enable Portal snapshots.

```text
Copyright (c) 2026 MonkeyKing.dev
```
