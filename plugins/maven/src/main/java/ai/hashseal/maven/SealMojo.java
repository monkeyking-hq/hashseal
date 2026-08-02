package ai.hashseal.maven;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import org.apache.maven.plugin.AbstractMojo;
import org.apache.maven.plugin.MojoExecutionException;
import org.apache.maven.plugin.MojoFailureException;
import org.apache.maven.plugins.annotations.LifecyclePhase;
import org.apache.maven.plugins.annotations.Mojo;
import org.apache.maven.plugins.annotations.Parameter;
import org.apache.maven.project.MavenProject;

/**
 * Run {@code hashseal seal} (instruct and/or tree).
 *
 * <pre>
 * mvn ai.hashseal:hashseal-maven-plugin:0.1.0-SNAPSHOT:seal
 * </pre>
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
@Mojo(name = "seal", defaultPhase = LifecyclePhase.PROCESS_RESOURCES, requiresProject = false, threadSafe = true)
public class SealMojo extends AbstractMojo {

  @Parameter(defaultValue = "${project}", readonly = true)
  private MavenProject project;

  /** Absolute path to hashseal binary; else HASHSEAL_BIN; else {@code hashseal} on PATH. */
  @Parameter(property = "hashseal.bin")
  private String bin;

  /** Project root passed as {@code --root}. Defaults to ${project.basedir} or cwd. */
  @Parameter(property = "hashseal.root")
  private File root;

  /** Seal instruct files ({@code --instruct}). Default true. */
  @Parameter(property = "hashseal.instruct", defaultValue = "true")
  private boolean instruct;

  /** Seal source tree ({@code --tree}). Default false. */
  @Parameter(property = "hashseal.tree", defaultValue = "false")
  private boolean tree;

  /** Emit integrity bundle ({@code --release}). */
  @Parameter(property = "hashseal.release", defaultValue = "false")
  private boolean release;

  /** GPG sign digests ({@code --sign}). */
  @Parameter(property = "hashseal.sign", defaultValue = "false")
  private boolean sign;

  /** Skip execution. */
  @Parameter(property = "hashseal.skip", defaultValue = "false")
  private boolean skip;

  @Override
  public void execute() throws MojoExecutionException, MojoFailureException {
    if (skip) {
      getLog().info("hashseal:seal skipped (hashseal.skip=true)");
      return;
    }
    if (!instruct && !tree) {
      throw new MojoExecutionException("Set hashseal.instruct and/or hashseal.tree to true");
    }

    File basedir = resolveRoot();
    String binary = HashsealCli.resolveBin(bin);
    List<String> args = new ArrayList<>();
    args.add("seal");
    if (instruct) {
      args.add("--instruct");
    }
    if (tree) {
      args.add("--tree");
    }
    if (release) {
      args.add("--release");
    }
    if (sign) {
      args.add("--sign");
    }
    args.add("--root");
    args.add(basedir.getAbsolutePath());

    getLog().info("Running: " + binary + " " + String.join(" ", args));
    try {
      int code = HashsealCli.run(binary, args, basedir, null);
      if (code != 0) {
        throw new MojoFailureException("hashseal seal failed with exit code " + code);
      }
    } catch (MojoFailureException e) {
      throw e;
    } catch (Exception e) {
      throw new MojoExecutionException(
          "Failed to run hashseal CLI ("
              + binary
              + "). Install hashseal and put it on PATH, or set -Dhashseal.bin=... / HASHSEAL_BIN. "
              + e.getMessage(),
          e);
    }
  }

  private File resolveRoot() {
    if (root != null) {
      return root;
    }
    if (project != null && project.getBasedir() != null) {
      return project.getBasedir();
    }
    return new File(System.getProperty("user.dir"));
  }
}
