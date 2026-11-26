/**
 * @hashseal/verify — zero-dependency HashSeal instruct + tree check.
 * Signed, Sealed, Delivered - I'm Yours.
 * Copyright (c) 2026 MonkeyKing.dev
 */

"use strict";

const {
  checkDocumentText,
  scanSuspiciousUnicode,
  blake3Hex,
  blake3Digest,
  SEAL_FIELD,
} = require("./check.js");

const { verifySshSig, parseAllowedSigners } = require("./sshsig.js");

const {
  hashTreeFileContent,
  verifyTreeInMemory,
  DEFAULT_TEXT_EXTENSIONS,
} = require("./tree.js");

module.exports = {
  checkDocumentText,
  scanSuspiciousUnicode,
  verifySshSig,
  parseAllowedSigners,
  blake3Hex,
  blake3Digest,
  SEAL_FIELD,
  hashTreeFileContent,
  verifyTreeInMemory,
  DEFAULT_TEXT_EXTENSIONS,
};
