/**
 * HashSeal browser popup — paste Markdown, pure-JS check, list non-OK results.
 * Copyright (c) 2026 MonkeyKing.dev
 */
"use strict";

const SAMPLE_VALID = `---
hashseal: "blake3:25280e93176b8b5ae3f4c2dd4b8fef7a20c4a626ea8dfd933b0e77b3a240dccb"
---
# Hello

Agent rules.
`;

const SAMPLE_MISMATCH = `---
title: demo
hashseal: "blake3:97bc50a158eeb098cd9ef101a8106b4dd3ae1585e16486d3e8761a54421e5fda"
---
# Demo

Evil.
`;

const docEl = document.getElementById("doc");
const summaryEl = document.getElementById("summary");
const resultsEl = document.getElementById("results");

function requireApi() {
  const api = globalThis.HashsealVerify;
  if (!api || typeof api.checkDocumentText !== "function") {
    throw new Error("HashsealVerify bundle not loaded");
  }
  return api;
}

/**
 * Split pasted text into logical documents.
 * Supports:
 * - single document
 * - multiple docs separated by a line of only ---DOC--- or ===
 * - multi-file paste with "# file: path" headers (optional)
 */
function splitDocuments(text) {
  const raw = String(text || "");
  if (!raw.trim()) return [];

  // Multi-file header: lines like `# file: AGENTS.md` then content until next header
  const fileHeader = /^#\s*file:\s*(.+)\s*$/im;
  if (fileHeader.test(raw)) {
    const parts = [];
    const lines = raw.split(/\r?\n/);
    let path = "pasted.md";
    let buf = [];
    const flush = () => {
      const body = buf.join("\n");
      if (body.trim()) parts.push({ path, text: body });
      buf = [];
    };
    for (const line of lines) {
      const m = line.match(/^#\s*file:\s*(.+)\s*$/i);
      if (m) {
        flush();
        path = m[1].trim() || "pasted.md";
        continue;
      }
      buf.push(line);
    }
    flush();
    return parts;
  }

  // Explicit multi-doc separator
  if (/^---DOC---\s*$/m.test(raw) || /^===\s*$/m.test(raw)) {
    return raw
      .split(/^---DOC---\s*$/m)
      .flatMap((chunk) => chunk.split(/^===\s*$/m))
      .map((t, i) => ({ path: `document-${i + 1}.md`, text: t }))
      .filter((d) => d.text.trim());
  }

  return [{ path: "pasted.md", text: raw }];
}

function renderResult(entry) {
  const { path, result } = entry;
  const card = document.createElement("article");
  card.className = "card";

  const pathEl = document.createElement("div");
  pathEl.className = "path";
  pathEl.textContent = path;
  card.appendChild(pathEl);

  const st = document.createElement("div");
  st.className = "status " + (result.status || "unknown");
  const label =
    result.status === "mismatch"
      ? "MISMATCH"
      : String(result.status || "unknown").toUpperCase();
  st.textContent = label;
  card.appendChild(st);

  const dl = document.createElement("dl");
  const rows = [
    ["ok", String(result.ok)],
    ["expected", result.expected || "—"],
    ["actual", result.actual || "—"],
  ];
  if (result.message) rows.push(["message", result.message]);
  if (result.algorithm) rows.push(["algorithm", result.algorithm]);
  for (const [k, v] of rows) {
    const dt = document.createElement("dt");
    dt.textContent = k;
    const dd = document.createElement("dd");
    dd.textContent = v;
    dl.appendChild(dt);
    dl.appendChild(dd);
  }
  card.appendChild(dl);
  return card;
}

function runCheck() {
  const api = requireApi();
  const docs = splitDocuments(docEl.value);
  resultsEl.innerHTML = "";
  summaryEl.hidden = false;

  if (docs.length === 0) {
    summaryEl.className = "summary warn";
    summaryEl.textContent = "Nothing to check — paste sealed Markdown.";
    return;
  }

  const entries = docs.map((d) => ({
    path: d.path,
    result: api.checkDocumentText(d.text),
  }));

  const failures = entries.filter((e) => !e.result.ok);
  const okCount = entries.length - failures.length;

  if (failures.length === 0) {
    summaryEl.className = "summary ok";
    summaryEl.innerHTML = "";
    const badge = document.createElement("img");
    badge.className = "summary__badge";
    badge.src = "icons/verified.svg";
    badge.width = 20;
    badge.height = 20;
    badge.alt = "";
    const text = document.createElement("span");
    text.className = "summary__text";
    text.textContent =
      entries.length === 1
        ? "Verified by HashSeal — seal matches content"
        : `Verified by HashSeal — all ${entries.length} document(s) VALID`;
    summaryEl.appendChild(badge);
    summaryEl.appendChild(text);
  } else {
    summaryEl.className = "summary bad";
    summaryEl.textContent =
      failures.length === entries.length
        ? `${failures.length} non-OK document(s) - full list below`
        : `${failures.length} non-OK / ${okCount} ok - every failure listed`;
  }

  // Verify UX: list every non-OK path; also show OK cards for multi-doc context
  const toShow =
    entries.length === 1 ? entries : failures.length ? failures : entries;
  for (const e of toShow) {
    resultsEl.appendChild(renderResult(e));
  }

  // If multi-doc and some ok hidden, note it
  if (entries.length > 1 && failures.length > 0 && okCount > 0) {
    const note = document.createElement("p");
    note.className = "hint";
    note.textContent = `${okCount} other document(s) OK (not listed).`;
    resultsEl.appendChild(note);
  }
}

document.getElementById("btn-check").addEventListener("click", () => {
  try {
    runCheck();
  } catch (err) {
    summaryEl.hidden = false;
    summaryEl.className = "summary bad";
    summaryEl.textContent = String(err && err.message ? err.message : err);
    resultsEl.innerHTML = "";
  }
});

document.getElementById("btn-clear").addEventListener("click", () => {
  docEl.value = "";
  summaryEl.hidden = true;
  resultsEl.innerHTML = "";
  docEl.focus();
});

document.getElementById("btn-sample").addEventListener("click", () => {
  // Toggle-ish: show valid then mismatch via multi-file so list is visible
  docEl.value =
    "# file: ok.md\n" +
    SAMPLE_VALID +
    "\n# file: tampered.md\n" +
    SAMPLE_MISMATCH;
  runCheck();
});
