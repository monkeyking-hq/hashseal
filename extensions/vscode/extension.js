/**
 * HashSeal VS Code extension — spawn hashseal / hashseal-check from PATH.
 * Zero runtime npm dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const vscode = require("vscode");
const { spawn } = require("child_process");
const path = require("path");
const fs = require("fs");

/**
 * @param {vscode.ExtensionContext} context
 */
function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand("hashseal.checkWorkspace", () =>
      runOnWorkspace("check")
    ),
    vscode.commands.registerCommand("hashseal.sealInstructWorkspace", () =>
      runOnWorkspace("seal-instruct")
    ),
    vscode.commands.registerCommand("hashseal.checkActiveFile", () =>
      checkActiveFile()
    )
  );
}

function deactivate() {}

/**
 * @returns {string | undefined}
 */
function workspaceRoot() {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) {
    vscode.window.showErrorMessage("HashSeal: open a workspace folder first.");
    return undefined;
  }
  return folders[0].uri.fsPath;
}

/**
 * Resolve full CLI binary.
 * @returns {string}
 */
function resolveHashsealBin() {
  const cfg = vscode.workspace.getConfiguration("hashseal");
  const configured = (cfg.get("bin") || "").trim();
  if (configured) return configured;
  if (process.env.HASHSEAL_BIN) return process.env.HASHSEAL_BIN;
  return process.platform === "win32" ? "hashseal.exe" : "hashseal";
}

/**
 * Resolve tiny check binary.
 * @returns {string}
 */
function resolveCheckBin() {
  const cfg = vscode.workspace.getConfiguration("hashseal");
  const configured = (cfg.get("checkBin") || "").trim();
  if (configured) return configured;
  if (process.env.HASHSEAL_CHECK_BIN) return process.env.HASHSEAL_CHECK_BIN;
  return process.platform === "win32" ? "hashseal-check.exe" : "hashseal-check";
}

/**
 * @param {"check"|"seal-instruct"} mode
 */
async function runOnWorkspace(mode) {
  const root = workspaceRoot();
  if (!root) return;

  const cfg = vscode.workspace.getConfiguration("hashseal");
  let bin;
  /** @type {string[]} */
  let args;

  if (mode === "check" && cfg.get("preferCheckBinary") !== false) {
    bin = resolveCheckBin();
    args = ["--root", root];
  } else if (mode === "check") {
    bin = resolveHashsealBin();
    args = ["check", "--root", root];
  } else {
    bin = resolveHashsealBin();
    args = ["seal", "--instruct", "--root", root];
  }

  const channel = getOutput();
  channel.show(true);
  channel.appendLine(`$ ${bin} ${args.join(" ")}`);
  channel.appendLine(`cwd: ${root}`);

  try {
    const code = await spawnInherit(bin, args, root, channel);
    if (code === 0) {
      vscode.window.showInformationMessage(
        mode === "check" ? "HashSeal check: OK" : "HashSeal seal: OK"
      );
    } else {
      vscode.window.showErrorMessage(
        `HashSeal exited ${code} (see HashSeal output for every non-OK path)`
      );
    }
  } catch (e) {
    const msg = e && e.message ? e.message : String(e);
    channel.appendLine(msg);
    vscode.window.showErrorMessage(
      `HashSeal failed to start (${bin}). Install CLI, set PATH / HASHSEAL_BIN / hashseal.bin. ${msg}`
    );
  }
}

/**
 * Check active editor document text via full CLI `check` if file is under root,
 * or report that in-editor WASM is not wired yet — spawn check on workspace is primary.
 * For a single open file path, run hashseal-check / check on workspace (CLI lists bad files).
 */
async function checkActiveFile() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage("HashSeal: no active editor.");
    return;
  }
  const doc = editor.document;
  if (doc.isUntitled) {
    vscode.window.showWarningMessage(
      "HashSeal: save the file first so the CLI can check it on disk."
    );
    return;
  }
  // Prefer workspace check so verify UX names every bad file under root.
  // Additionally echo the active path for focus.
  const channel = getOutput();
  channel.show(true);
  channel.appendLine(`Active file: ${doc.uri.fsPath}`);
  await runOnWorkspace("check");
}

/**
 * @param {string} bin
 * @param {string[]} args
 * @param {string} cwd
 * @param {vscode.OutputChannel} channel
 * @returns {Promise<number>}
 */
function spawnInherit(bin, args, cwd, channel) {
  return new Promise((resolve, reject) => {
    const child = spawn(bin, args, {
      cwd,
      env: process.env,
      shell: false,
      windowsHide: true,
    });
    child.stdout.on("data", (d) => channel.append(d.toString()));
    child.stderr.on("data", (d) => channel.append(d.toString()));
    child.on("error", (err) => {
      if (err.code === "ENOENT") {
        err.message =
          `CLI not found: "${bin}". Put hashseal / hashseal-check on PATH, ` +
          "or set HASHSEAL_BIN / HASHSEAL_CHECK_BIN / settings hashseal.bin.";
      }
      reject(err);
    });
    child.on("close", (code) => resolve(code == null ? 1 : code));
  });
}

/** @type {vscode.OutputChannel | undefined} */
let outputChannel;

function getOutput() {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("HashSeal");
  }
  return outputChannel;
}

module.exports = { activate, deactivate };
