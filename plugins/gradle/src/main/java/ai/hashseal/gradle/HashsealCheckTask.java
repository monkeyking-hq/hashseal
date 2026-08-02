package ai.hashseal.gradle;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import org.gradle.api.DefaultTask;
import org.gradle.api.GradleException;
import org.gradle.api.file.DirectoryProperty;
import org.gradle.api.provider.Property;
import org.gradle.api.tasks.Input;
import org.gradle.api.tasks.InputDirectory;
import org.gradle.api.tasks.Optional;
import org.gradle.api.tasks.PathSensitive;
import org.gradle.api.tasks.PathSensitivity;
import org.gradle.api.tasks.TaskAction;

/**
 * Run {@code hashseal check} — fails the build when digests mismatch. CLI lists every non-OK path
 * (verify UX).
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
public abstract class HashsealCheckTask extends DefaultTask {

  @Input
  @Optional
  public abstract Property<String> getBin();

  @InputDirectory
  @PathSensitive(PathSensitivity.RELATIVE)
  public abstract DirectoryProperty getRoot();

  @Input
  public abstract Property<Boolean> getRequireSignature();

  @Input
  public abstract Property<Boolean> getSkip();

  @TaskAction
  public void runCheck() {
    if (Boolean.TRUE.equals(getSkip().getOrElse(false))) {
      getLogger().lifecycle("hashsealCheck skipped (hashseal.skip=true)");
      return;
    }

    File basedir = getRoot().get().getAsFile();
    String binary = HashsealCli.resolveBin(getBin().getOrNull());
    List<String> args = new ArrayList<>();
    args.add("check");
    if (Boolean.TRUE.equals(getRequireSignature().getOrElse(false))) {
      args.add("--require-signature");
    }
    args.add("--root");
    args.add(basedir.getAbsolutePath());

    getLogger().lifecycle("Running: " + binary + " " + String.join(" ", args));
    try {
      int code = HashsealCli.run(binary, args, basedir, null);
      if (code != 0) {
        throw new GradleException(
            "hashseal check failed with exit code "
                + code
                + " (CLI lists every non-OK path)");
      }
    } catch (GradleException e) {
      throw e;
    } catch (Exception e) {
      throw new GradleException(
          "Failed to run hashseal CLI ("
              + binary
              + "). Install hashseal and put it on PATH, or set HASHSEAL_BIN / hashseal.bin. "
              + e.getMessage(),
          e);
    }
  }
}
