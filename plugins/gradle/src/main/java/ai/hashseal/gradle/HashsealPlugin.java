package ai.hashseal.gradle;

import org.gradle.api.Plugin;
import org.gradle.api.Project;
import org.gradle.api.tasks.Exec;

/**
 * Registers HashSeal tasks that shell out to the {@code hashseal} CLI.
 *
 * <p>Binary resolution: extension {@code bin}, then env {@code HASHSEAL_BIN}, then {@code
 * hashseal} on PATH.
 *
 * <p>Copyright (c) 2026 MonkeyKing.dev
 */
public class HashsealPlugin implements Plugin<Project> {

  @Override
  public void apply(Project project) {
    HashsealExtension ext =
        project.getExtensions().create("hashseal", HashsealExtension.class, project);

    project
        .getTasks()
        .register(
            "hashsealSealInstruct",
            Exec.class,
            task -> {
              task.setGroup("hashseal");
              task.setDescription("hashseal seal --instruct");
              task.doFirst(
                  t -> {
                    task.setExecutable(ext.resolveBin());
                    task.setArgs(ext.args("seal", "--instruct"));
                    task.setWorkingDir(ext.rootDir());
                  });
            });

    project
        .getTasks()
        .register(
            "hashsealCheck",
            Exec.class,
            task -> {
              task.setGroup("hashseal");
              task.setDescription("hashseal check");
              task.doFirst(
                  t -> {
                    task.setExecutable(ext.resolveBin());
                    task.setArgs(ext.args("check"));
                    task.setWorkingDir(ext.rootDir());
                  });
            });

    project
        .getTasks()
        .register(
            "hashsealVerify",
            Exec.class,
            task -> {
              task.setGroup("hashseal");
              task.setDescription("hashseal verify");
              task.doFirst(
                  t -> {
                    task.setExecutable(ext.resolveBin());
                    task.setArgs(ext.args("verify"));
                    task.setWorkingDir(ext.rootDir());
                  });
            });

    project
        .getTasks()
        .register(
            "hashsealSealTree",
            Exec.class,
            task -> {
              task.setGroup("hashseal");
              task.setDescription("hashseal seal --tree --release");
              task.doFirst(
                  t -> {
                    task.setExecutable(ext.resolveBin());
                    task.setArgs(ext.args("seal", "--tree", "--release"));
                    task.setWorkingDir(ext.rootDir());
                  });
            });
  }
}
