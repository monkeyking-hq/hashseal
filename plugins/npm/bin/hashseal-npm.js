#!/usr/bin/env node
/**
 * CLI entry: hashseal-npm <hashseal-args...>
 * Forwards all args to the hashseal binary on PATH / HASHSEAL_BIN.
 *
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const { runHashseal, resolveHashsealBin } = require("../index.js");

const args = process.argv.slice(2);
if (args.length === 0 || args[0] === "-h" || args[0] === "--help") {
  console.log(`hashseal-npm — thin wrapper for the HashSeal CLI

Usage:
  hashseal-npm <hashseal-subcommand> [args...]

Examples:
  hashseal-npm seal --instruct --root .
  hashseal-npm check --root .
  hashseal-npm verify --root .

Requires the \`hashseal\` binary on PATH, or set HASHSEAL_BIN.
Current binary: ${resolveHashsealBin()}

Copyright (c) 2026 MonkeyKing.dev
`);
  process.exit(0);
}

const result = runHashseal(args, { stdio: "inherit" });
if (result.error) {
  console.error(result.error.message);
  process.exit(127);
}
process.exit(result.status == null ? 1 : result.status);
