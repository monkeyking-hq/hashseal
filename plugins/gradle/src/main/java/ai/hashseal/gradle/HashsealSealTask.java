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
 * Run {@code hashseal seal} (instruct and/or tree).
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
public abstract class HashsealSealTask extends DefaultTask {

  @Input
  @Optional
  public abstract Property<String> getBin();

  @InputDirectory
  @PathSensitive(PathSensitivity.RELATIVE)
  public abstract DirectoryProperty getRoot();

  @Input
  public abstract Property<Boolean> getInstruct();

  @Input
  public abstract Property<Boolean> getTree();

  @Input
  public abstract Property<Boolean> getRelease();

  @Input
  public abstract Property<Boolean> getSign();

  @Input
  public abstract Property<Boolean> getSkip();

  @TaskAction
  public void runSeal() {
    if (Boolean.TRUE.equals(getSkip().getOrElse(false))) {
      getLogger().lifecycle("hashsealSeal skipped (hashseal.skip=true)");
      return;
    }
    boolean instruct = Boolean.TRUE.equals(getInstruct().getOrElse(true));
    boolean tree = Boolean.TRUE.equals(getTree().getOrElse(false));
    if (!instruct && !tree) {
      throw new GradleException("Set hashseal.instruct and/or hashseal.tree to true");
    }

    File basedir = getRoot().get().getAsFile();
    String binary = HashsealCli.resolveBin(getBin().getOrNull());
    List<String> args = new ArrayList<>();
    args.add("seal");
    if (instruct) {
      args.add("--instruct");
    }
    if (tree) {
      args.add("--tree");
    }
    if (Boolean.TRUE.equals(getRelease().getOrElse(false))) {
      args.add("--release");
    }
    if (Boolean.TRUE.equals(getSign().getOrElse(false))) {
      args.add("--sign");
    }
    args.add("--root");
    args.add(basedir.getAbsolutePath());

    getLogger().lifecycle("Running: " + binary + " " + String.join(" ", args));
    try {
      int code = HashsealCli.run(binary, args, basedir, null);
      if (code != 0) {
        throw new GradleException("hashseal seal failed with exit code " + code);
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
