package ai.hashseal.maven;

import java.io.File;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;

/**
 * Shared process launcher for the hashseal CLI.
 *
 * Resolution order for the binary:
 * 1. explicit {@code bin} parameter
 * 2. environment variable {@code HASHSEAL_BIN}
 * 3. bare command {@code hashseal} on PATH
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */
final class HashsealCli {
  private HashsealCli() {}

  static String resolveBin(String configuredBin) {
    if (configuredBin != null && !configuredBin.isBlank()) {
      return configuredBin.trim();
    }
    String env = System.getenv("HASHSEAL_BIN");
    if (env != null && !env.isBlank()) {
      return env.trim();
    }
    return "hashseal";
  }

  static int run(String bin, List<String> args, File workingDirectory, Map<String, String> extraEnv)
      throws IOException, InterruptedException {
    List<String> command = new ArrayList<>();
    command.add(bin);
    command.addAll(args);

    ProcessBuilder pb = new ProcessBuilder(command);
    if (workingDirectory != null) {
      pb.directory(workingDirectory);
    }
    pb.inheritIO();
    if (extraEnv != null && !extraEnv.isEmpty()) {
      pb.environment().putAll(extraEnv);
    }
    Process p = pb.start();
    return p.waitFor();
  }
}
