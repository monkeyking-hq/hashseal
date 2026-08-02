/**
 * @hashseal/npm-plugin — thin wrapper around the `hashseal` CLI.
 * Requires `hashseal` on PATH (or HASHSEAL_BIN).
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { spawnSync } = require("child_process");
const path = require("path");

/**
 * Resolve the hashseal CLI binary.
 * @returns {string}
 */
function resolveHashsealBin() {
  if (process.env.HASHSEAL_BIN) {
    return process.env.HASHSEAL_BIN;
  }
  // Default: bare command on PATH (Windows: hashseal.exe via PATHEXT)
  return "hashseal";
}

/**
 * Run `hashseal` with argv (excluding the program name).
 * @param {string[]} args
 * @param {{ cwd?: string, stdio?: import('child_process').StdioOptions, env?: NodeJS.ProcessEnv }} [opts]
 * @returns {{ status: number|null, signal: string|null, error?: Error, stdout: string, stderr: string }}
 */
function runHashseal(args, opts) {
  const bin = resolveHashsealBin();
  const cwd = (opts && opts.cwd) || process.cwd();
  const stdio = (opts && opts.stdio) || "pipe";
  const env = Object.assign({}, process.env, (opts && opts.env) || {});

  const result = spawnSync(bin, args, {
    cwd,
    env,
    encoding: "utf8",
    shell: false,
    stdio,
  });

  if (result.error) {
    const err = result.error;
    if (err.code === "ENOENT") {
      err.message =
        `hashseal CLI not found (looked for "${bin}"). ` +
        "Install hashseal and ensure it is on PATH, or set HASHSEAL_BIN to the binary path. " +
        "See plugins/npm/README.md.";
    }
    return {
      status: null,
      signal: null,
      error: err,
      stdout: result.stdout || "",
      stderr: result.stderr || "",
    };
  }

  return {
    status: result.status,
    signal: result.signal,
    stdout: result.stdout || "",
    stderr: result.stderr || "",
  };
}

/**
 * Seal instruct files under root (default: cwd).
 * @param {{ root?: string, sign?: boolean, extraArgs?: string[] }} [opts]
 */
function sealInstruct(opts) {
  opts = opts || {};
  const args = ["seal", "--instruct"];
  if (opts.root) {
    args.push("--root", opts.root);
  }
  if (opts.sign) {
    args.push("--sign");
  }
  if (opts.extraArgs) {
    args.push(...opts.extraArgs);
  }
  return runHashseal(args, { cwd: opts.root ? path.resolve(opts.root, "..") : undefined });
}

/**
 * Check sealed instruct / tree under root.
 * @param {{ root?: string, extraArgs?: string[] }} [opts]
 */
function check(opts) {
  opts = opts || {};
  const args = ["check"];
  if (opts.root) {
    args.push("--root", opts.root);
  }
  if (opts.extraArgs) {
    args.push(...opts.extraArgs);
  }
  return runHashseal(args);
}

/**
 * Verify integrity bundle under root.
 * @param {{ root?: string, extraArgs?: string[] }} [opts]
 */
function verify(opts) {
  opts = opts || {};
  const args = ["verify"];
  if (opts.root) {
    args.push("--root", opts.root);
  }
  if (opts.extraArgs) {
    args.push(...opts.extraArgs);
  }
  return runHashseal(args);
}

module.exports = {
  resolveHashsealBin,
  runHashseal,
  sealInstruct,
  check,
  verify,
};
