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
 * Run {@code hashseal check} — fails the build when digests mismatch.
 * CLI lists every non-OK path (verify UX).
 *
 * <pre>
 * mvn ai.hashseal:hashseal-maven-plugin:0.1.0-SNAPSHOT:check
 * </pre>
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
@Mojo(name = "check", defaultPhase = LifecyclePhase.VERIFY, requiresProject = false, threadSafe = true)
public class CheckMojo extends AbstractMojo {

  @Parameter(defaultValue = "${project}", readonly = true)
  private MavenProject project;

  @Parameter(property = "hashseal.bin")
  private String bin;

  @Parameter(property = "hashseal.root")
  private File root;

  @Parameter(property = "hashseal.requireSignature", defaultValue = "false")
  private boolean requireSignature;

  @Parameter(property = "hashseal.skip", defaultValue = "false")
  private boolean skip;

  @Override
  public void execute() throws MojoExecutionException, MojoFailureException {
    if (skip) {
      getLog().info("hashseal:check skipped (hashseal.skip=true)");
      return;
    }

    File basedir = resolveRoot();
    String binary = HashsealCli.resolveBin(bin);
    List<String> args = new ArrayList<>();
    args.add("check");
    if (requireSignature) {
      args.add("--require-signature");
    }
    args.add("--root");
    args.add(basedir.getAbsolutePath());

    getLog().info("Running: " + binary + " " + String.join(" ", args));
    try {
      int code = HashsealCli.run(binary, args, basedir, null);
      if (code != 0) {
        throw new MojoFailureException(
            "hashseal check failed with exit code "
                + code
                + " (CLI lists every non-OK path)");
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
