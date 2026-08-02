package ai.hashseal.gradle;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import javax.inject.Inject;
import org.gradle.api.Project;
import org.gradle.api.provider.Property;

/**
 * HashSeal plugin extension.
 *
 * <p>Copyright (c) 2026 MonkeyKing.dev
 */
public abstract class HashsealExtension {

  private final Project project;

  @Inject
  public HashsealExtension(Project project) {
    this.project = project;
    getBin().convention("");
    getRoot().convention(project.getLayout().getProjectDirectory().getAsFile().getAbsolutePath());
    getSign().convention(false);
  }

  /** Explicit path to hashseal binary; empty = HASHSEAL_BIN or PATH. */
  public abstract Property<String> getBin();

  /** Project root passed as {@code --root}. */
  public abstract Property<String> getRoot();

  /** When true, {@code seal --instruct} also passes {@code --sign}. */
  public abstract Property<Boolean> getSign();

  String resolveBin() {
    String configured = getBin().getOrElse("");
    if (configured != null && !configured.isBlank()) {
      return configured.trim();
    }
    String env = System.getenv("HASHSEAL_BIN");
    if (env != null && !env.isBlank()) {
      return env.trim();
    }
    return "hashseal";
  }

  File rootDir() {
    return new File(getRoot().get());
  }

  List<String> args(String... command) {
    List<String> a = new ArrayList<>();
    for (String c : command) {
      a.add(c);
    }
    a.add("--root");
    a.add(getRoot().get());
    if (getSign().getOrElse(false) && command.length > 0 && "seal".equals(command[0])) {
      if (!a.contains("--sign")) {
        a.add("--sign");
      }
    }
    return a;
  }
}
