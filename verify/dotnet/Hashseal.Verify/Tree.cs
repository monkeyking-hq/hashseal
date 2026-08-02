// In-memory tree verify — mirrors hashseal-core / verify/js tree.js.
// Zero NuGet dependencies.
//
// Copyright (c) 2026 MonkeyKing.dev

using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace Hashseal.Verify
{
    public sealed class TreeHashResult
    {
        public string Digest { get; init; } = "";
        public string Qualified { get; init; } = "";
        public string Hex { get; init; } = "";
        public int Size { get; init; }
    }

    public sealed class TreeFinding
    {
        public string Path { get; init; } = "";
        public string Status { get; init; } = "";
        public string? Expected { get; init; }
        public string? Actual { get; init; }
    }

    public sealed class TreeVerifyResult
    {
        public bool Ok { get; init; }
        public int Checked { get; init; }
        public List<TreeFinding> Findings { get; init; } = new();
    }

    public static class Tree
    {
        public static readonly HashSet<string> DefaultTextExtensions = new(StringComparer.OrdinalIgnoreCase)
        {
            "md", "txt", "toml", "yml", "yaml", "json", "rs", "java", "go", "py", "js",
            "ts", "tsx", "jsx", "css", "html", "xml", "sh", "ps1", "c", "h", "cpp", "cs", "rb", "svg"
        };

        public static string NormalizeLf(string s) =>
            s.Replace("\r\n", "\n", StringComparison.Ordinal).Replace("\r", "\n", StringComparison.Ordinal);

        public static string ExtOf(string path)
        {
            var i = path.LastIndexOf('.');
            if (i < 0 || i == path.Length - 1) return "";
            return path[(i + 1)..].ToLowerInvariant();
        }

        /// <summary>Hash one path+content with core tree policy. Size is UTF-8 bytes before normalize.</summary>
        public static TreeHashResult HashTreeFileContent(
            string path,
            string content,
            bool lineEndingsLfText = true,
            HashSet<string>? textExtensions = null)
        {
            textExtensions ??= DefaultTextExtensions;
            content ??= "";
            var size = Encoding.UTF8.GetByteCount(content);
            var data = content;
            if (lineEndingsLfText && textExtensions.Contains(ExtOf(path)))
            {
                if (data.Length > 0 && data[0] == '\uFEFF')
                    data = data[1..];
                data = NormalizeLf(data);
            }
            var hex = Blake3.HashHex(Encoding.UTF8.GetBytes(data));
            var qualified = "blake3:" + hex;
            return new TreeHashResult
            {
                Digest = qualified,
                Qualified = qualified,
                Hex = hex,
                Size = size
            };
        }

        /// <summary>Verify in-memory files against ledger entries.</summary>
        public static TreeVerifyResult VerifyTreeInMemory(
            IReadOnlyDictionary<string, string>? files,
            IReadOnlyList<LedgerEntryLike>? ledgerEntries,
            bool lineEndingsLfText = true,
            HashSet<string>? textExtensions = null)
        {
            files ??= new Dictionary<string, string>();
            var current = new SortedDictionary<string, string>(StringComparer.Ordinal);
            foreach (var p in files.Keys.OrderBy(x => x, StringComparer.Ordinal))
            {
                var h = HashTreeFileContent(p, files[p], lineEndingsLfText, textExtensions);
                current[p] = h.Qualified;
            }

            var findings = new List<TreeFinding>();
            var expectedPaths = new HashSet<string>(StringComparer.Ordinal);
            var entries = ledgerEntries ?? Array.Empty<LedgerEntryLike>();

            foreach (var e in entries)
            {
                expectedPaths.Add(e.Path);
                if (!current.TryGetValue(e.Path, out var actual))
                {
                    findings.Add(new TreeFinding
                    {
                        Path = e.Path,
                        Status = "removed",
                        Expected = e.Digest,
                        Actual = null
                    });
                }
                else if (actual != e.Digest)
                {
                    findings.Add(new TreeFinding
                    {
                        Path = e.Path,
                        Status = "mismatch",
                        Expected = e.Digest,
                        Actual = actual
                    });
                }
            }

            foreach (var path in current.Keys)
            {
                if (expectedPaths.Contains(path)) continue;
                findings.Add(new TreeFinding
                {
                    Path = path,
                    Status = "added",
                    Expected = null,
                    Actual = current[path]
                });
            }

            findings.Sort((a, b) => string.CompareOrdinal(a.Path, b.Path));
            return new TreeVerifyResult
            {
                Ok = findings.Count == 0,
                Checked = entries.Count,
                Findings = findings
            };
        }
    }

    public sealed class LedgerEntryLike
    {
        public string Path { get; init; } = "";
        public string Digest { get; init; } = "";
    }
}
