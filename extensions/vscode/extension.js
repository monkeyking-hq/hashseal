/**
 * HashSeal VS Code extension — diagnostics, status bar, check-on-save, and single-file check.
 * Zero runtime npm dependencies.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const vscode = require("vscode");
const { spawn } = require("child_process");
const path = require("path");

/** @type {vscode.DiagnosticCollection} */
let diagnostics;
/** @type {vscode.StatusBarItem} */
let statusBar;
/** @type {vscode.OutputChannel | undefined} */
let outputChannel;

function activate(context) {
  diagnostics = vscode.languages.createDiagnosticCollection("hashseal");
  statusBar = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 50);
  statusBar.command = "hashseal.checkWorkspace";
  statusBar.text = "$(shield) HashSeal";
  statusBar.tooltip = "Click to run HashSeal workspace check";
  statusBar.show();

  context.subscriptions.push(
    diagnostics,
    statusBar,
    vscode.commands.registerCommand("hashseal.checkWorkspace", () =>
      runWorkspaceCheck()
    ),
    vscode.commands.registerCommand("hashseal.sealInstructWorkspace", () =>
      runSealWorkspace()
    ),
    vscode.commands.registerCommand("hashseal.checkActiveFile", () =>
      checkActiveFile()
    ),
    vscode.workspace.onDidSaveTextDocument((doc) => {
      const cfg = vscode.workspace.getConfiguration("hashseal");
      if (cfg.get("checkOnSave") !== false && isCandidateDoc(doc)) {
        runWorkspaceCheck({ quiet: true });
      }
    })
  );
}

function deactivate() {}

function isCandidateDoc(doc) {
  if (!doc || doc.isUntitled) return false;
  const p = doc.uri.fsPath.toLowerCase();
  return (
    p.endsWith(".md") ||
    p.endsWith(".mdc") ||
    p.endsWith(".cursorrules") ||
    p.endsWith(".windsurfrules") ||
    p.endsWith(".clinerules")
  );
}

function workspaceRoot() {
  const folders = vscode.workspace.workspaceFolders;
  if (!folders || folders.length === 0) {
    vscode.window.showErrorMessage("HashSeal: open a workspace folder first.");
    return undefined;
  }
  return folders[0].uri.fsPath;
}

function resolveHashsealBin() {
  const cfg = vscode.workspace.getConfiguration("hashseal");
  const configured = (cfg.get("bin") || "").trim();
  if (configured) return configured;
  if (process.env.HASHSEAL_BIN) return process.env.HASHSEAL_BIN;
  return process.platform === "win32" ? "hashseal.exe" : "hashseal";
}

function getOutput() {
  if (!outputChannel) {
    outputChannel = vscode.window.createOutputChannel("HashSeal");
  }
  return outputChannel;
}

async function runWorkspaceCheck(opts) {
  const quiet = opts && opts.quiet;
  const root = workspaceRoot();
  if (!root) return;

  const cfg = vscode.workspace.getConfiguration("hashseal");
  const bin = resolveHashsealBin();
  const args = ["check", "--format", "json", "--root", root];
  if (cfg.get("requireSignature")) args.push("--require-signature");

  try {
    const res = await spawnCapture(bin, args, root);
    const batch = JSON.parse(res.stdout);
    diagnostics.clear();

    if (batch.ok) {
      statusBar.text = `$(verified) HashSeal: ${batch.summary.ok} OK`;
      statusBar.backgroundColor = undefined;
      if (!quiet) {
        vscode.window.showInformationMessage(`HashSeal check: ${batch.summary.ok} file(s) OK`);
      }
    } else {
      const byFile = new Map();
      for (const f of batch.findings || []) {
        const fp = f.path ? (path.isAbsolute(f.path) ? f.path : path.join(root, f.path)) : root;
        const msg = `[${String(f.status || "error").toUpperCase()}] ${f.message || ""}` +
          (f.expected ? ` expected=${f.expected}` : "") +
          (f.actual ? ` actual=${f.actual}` : "");
        const diag = new vscode.Diagnostic(
          new vscode.Range(0, 0, 0, 1),
          msg.trim(),
          vscode.DiagnosticSeverity.Error
        );
        diag.source = "hashseal";
        if (!byFile.has(fp)) byFile.set(fp, []);
        byFile.get(fp).push(diag);
      }
      for (const [fp, list] of byFile.entries()) {
        diagnostics.set(vscode.Uri.file(fp), list);
      }
      statusBar.text = `$(error) HashSeal: ${batch.findings.length} issue(s)`;
      statusBar.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
      if (!quiet) {
        vscode.window.showErrorMessage(`HashSeal failed: ${batch.findings.length} issue(s)`);
      }
    }
  } catch (e) {
    statusBar.text = "$(warning) HashSeal: CLI error";
    if (!quiet) {
      vscode.window.showErrorMessage(`HashSeal failed: ${e.message || e}`);
    }
  }
}

async function checkActiveFile() {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    vscode.window.showErrorMessage("HashSeal: no active editor.");
    return;
  }
  const doc = editor.document;
  if (doc.isUntitled) {
    vscode.window.showWarningMessage("HashSeal: save the file first so the CLI can check it.");
    return;
  }
  const root = workspaceRoot() || path.dirname(doc.uri.fsPath);
  const cfg = vscode.workspace.getConfiguration("hashseal");
  const bin = resolveHashsealBin();
  const args = ["check", "--format", "json", "--root", root, doc.uri.fsPath];
  if (cfg.get("requireSignature")) args.push("--require-signature");

  try {
    const res = await spawnCapture(bin, args, root);
    const batch = JSON.parse(res.stdout);
    const uri = doc.uri;
    if (batch.ok) {
      diagnostics.delete(uri);
      vscode.window.showInformationMessage(`HashSeal: ${path.basename(uri.fsPath)} OK`);
    } else {
      const diags = (batch.findings || []).map((f) => {
        const d = new vscode.Diagnostic(
          new vscode.Range(0, 0, 0, 1),
          `[${String(f.status).toUpperCase()}] ${f.message || ""}`.trim(),
          vscode.DiagnosticSeverity.Error
        );
        d.source = "hashseal";
        return d;
      });
      diagnostics.set(uri, diags);
      vscode.window.showErrorMessage(`HashSeal: ${path.basename(uri.fsPath)} failed (${diags.length} issue)`);
    }
  } catch (e) {
    vscode.window.showErrorMessage(`HashSeal failed: ${e.message || e}`);
  }
}

async function runSealWorkspace() {
  const root = workspaceRoot();
  if (!root) return;
  const bin = resolveHashsealBin();
  const channel = getOutput();
  channel.show(true);
  channel.appendLine(`$ ${bin} seal --instruct --root ${root}`);
  try {
    const res = await spawnCapture(bin, ["seal", "--instruct", "--root", root], root);
    if (res.stdout) channel.append(res.stdout);
    if (res.stderr) channel.append(res.stderr);
    if (res.code === 0) {
      await runWorkspaceCheck({ quiet: true });
      vscode.window.showInformationMessage("HashSeal seal: OK");
    } else {
      vscode.window.showErrorMessage(`HashSeal seal exited ${res.code}`);
    }
  } catch (e) {
    vscode.window.showErrorMessage(`HashSeal failed: ${e.message || e}`);
  }
}

function spawnCapture(bin, args, cwd) {
  return new Promise((resolve, reject) => {
    const child = spawn(bin, args, { cwd, env: process.env, shell: false, windowsHide: true });
    let stdout = "";
    let stderr = "";
    child.stdout.on("data", (d) => { stdout += d.toString(); });
    child.stderr.on("data", (d) => { stderr += d.toString(); });
    child.on("error", reject);
    child.on("close", (code) => resolve({ code: code == null ? 1 : code, stdout, stderr }));
  });
}

module.exports = { activate, deactivate };
