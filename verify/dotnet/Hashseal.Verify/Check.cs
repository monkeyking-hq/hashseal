// HashSeal instruct document check — FULL canonical mode (digest only).
// Mirrors hashseal-core instruct algorithm. Zero NuGet dependencies.
//
// Copyright (c) 2026 MonkeyKing.dev

using System;
using System.Collections.Generic;
using System.Text;

namespace Hashseal.Verify
{
    /// <summary>Instruct document digest check (FULL canonical mode).</summary>
    public static class Check
    {
        public const string SealField = "hashseal";
        public const string SigField = "hashseal_sig";
        public const string KeyIdField = "hashseal_key_id";

        public sealed class Result
        {
            public bool Ok { get; set; }
            public string Status { get; set; }
            public string Algorithm { get; set; }
            public string Expected { get; set; }
            public string Actual { get; set; }
            public string Message { get; set; }

            public override string ToString()
            {
                return $"Result{{ok={Ok}, status={Status}, algorithm={Algorithm}, expected={Expected}, actual={Actual}, message={Message}}}";
            }
        }

        public sealed class Digest
        {
            public string Algorithm { get; set; }
            public string Hex { get; set; }
            public string Qualified { get; set; }
        }

        public static Result CheckDocumentText(string text)
        {
            return CheckDocumentText(text, SealField);
        }

        public static Result CheckDocumentText(string text, string field)
        {
            var doc = ParseDocument(text);
            if (!doc.HadFrontMatter)
            {
                var actual = ComputeDigest(doc);
                return new Result
                {
                    Ok = false,
                    Status = "missing_seal",
                    Algorithm = "blake3",
                    Expected = null,
                    Actual = actual.Qualified,
                    Message = "missing hashseal field"
                };
            }
            string sealRaw = ExtractReservedField(doc.FmLines, field);
            if (sealRaw == null)
            {
                var actual = ComputeDigest(doc);
                return new Result
                {
                    Ok = false,
                    Status = "missing_seal",
                    Algorithm = "blake3",
                    Expected = null,
                    Actual = actual.Qualified,
                    Message = "missing hashseal field"
                };
            }
            var expected = ParseDigest(sealRaw);
            if (expected == null)
            {
                return new Result
                {
                    Ok = false,
                    Status = "invalid_format",
                    Message = "invalid digest format: " + sealRaw
                };
            }
            if (expected.Algorithm != "blake3")
            {
                return new Result
                {
                    Ok = false,
                    Status = "invalid_format",
                    Algorithm = expected.Algorithm,
                    Expected = expected.Qualified,
                    Message = "unsupported algorithm: " + expected.Algorithm
                };
            }
            var actualDigest = ComputeDigest(doc);
            if (actualDigest.Hex != expected.Hex || actualDigest.Algorithm != expected.Algorithm)
            {
                return new Result
                {
                    Ok = false,
                    Status = "mismatch",
                    Algorithm = expected.Algorithm,
                    Expected = expected.Qualified,
                    Actual = actualDigest.Qualified
                };
            }
            return new Result
            {
                Ok = true,
                Status = "valid",
                Algorithm = actualDigest.Algorithm,
                Expected = expected.Qualified,
                Actual = actualDigest.Qualified
            };
        }

        public static Digest Blake3Digest(byte[] data)
        {
            string hex = Blake3.HashHex(data);
            return new Digest { Algorithm = "blake3", Hex = hex, Qualified = "blake3:" + hex };
        }

        public static Digest Blake3Digest(string utf8)
        {
            return Blake3Digest(Encoding.UTF8.GetBytes(utf8));
        }

        private sealed class ParsedDoc
        {
            public List<string> FmLines;
            public bool HadFrontMatter;
            public string BodyRaw;
        }

        private static string StripBom(string s)
        {
            if (!string.IsNullOrEmpty(s) && s[0] == '\uFEFF')
            {
                return s.Substring(1);
            }
            return s;
        }

        private static string NormalizeLf(string s)
        {
            return s.Replace("\r\n", "\n").Replace("\r", "\n");
        }

        private static ParsedDoc ParseDocument(string text)
        {
            text = StripBom(text);
            if (text.StartsWith("---\n") || text.StartsWith("---\r\n"))
            {
                string afterOpen = text.StartsWith("---\r\n") ? text.Substring(5) : text.Substring(4);
                string search = afterOpen;
                int offset = 0;
                while (true)
                {
                    int idx = search.IndexOf("\n---", StringComparison.Ordinal);
                    if (idx < 0) break;
                    string after = search.Substring(idx + 1);
                    string rest = after.Substring(3);
                    bool closed =
                        rest.Length == 0
                        || rest.StartsWith("\n")
                        || rest.StartsWith("\r\n")
                        || rest.StartsWith("\r");
                    if (closed)
                    {
                        string fmBlock = afterOpen.Substring(0, offset + idx);
                        string body = afterOpen.Substring(idx + 1 + 3);
                        if (body.StartsWith("\r\n")) body = body.Substring(2);
                        else if (body.StartsWith("\n")) body = body.Substring(1);
                        else if (body.StartsWith("\r")) body = body.Substring(1);
                        string[] parts = NormalizeLf(fmBlock).Split(new[] { '\n' }, StringSplitOptions.None);
                        var fmLines = new List<string>(parts);
                        return new ParsedDoc { FmLines = fmLines, HadFrontMatter = true, BodyRaw = body };
                    }
                    offset += idx + 1;
                    search = search.Substring(idx + 1);
                }
            }
            return new ParsedDoc { FmLines = new List<string>(), HadFrontMatter = false, BodyRaw = text };
        }

        private static bool IsReservedKey(string key)
        {
            return key == SealField || key == SigField || key == KeyIdField;
        }

        private static string TrimLeadingSpaceTab(string s)
        {
            int j = 0;
            while (j < s.Length && (s[j] == ' ' || s[j] == '\t')) j++;
            return s.Substring(j);
        }

        private static void ForEachFmEntry(List<string> lines, Action<string, string> f)
        {
            int i = 0;
            int n = lines.Count;
            while (i < n)
            {
                string line = lines[i];
                string trimmed = line.Trim();
                if (trimmed.Length == 0 || trimmed.StartsWith("#"))
                {
                    i++;
                    continue;
                }
                if (line.StartsWith(" ") || line.StartsWith("\t"))
                {
                    i++;
                    continue;
                }
                int colon = trimmed.IndexOf(':');
                if (colon >= 0)
                {
                    string key = trimmed.Substring(0, colon).Trim();
                    string rest = trimmed.Substring(colon + 1).Trim();
                    if (IsReservedKey(key))
                    {
                        i++;
                        while (i < n)
                        {
                            string L = lines[i];
                            if (L.StartsWith(" ") || L.StartsWith("\t"))
                            {
                                i++;
                                continue;
                            }
                            if (L.Trim().Length == 0)
                            {
                                if (i + 1 < n && (lines[i + 1].StartsWith(" ") || lines[i + 1].StartsWith("\t")))
                                {
                                    i++;
                                    continue;
                                }
                                break;
                            }
                            break;
                        }
                        continue;
                    }
                    if (rest == "|" || rest == ">" || rest == "|-" || rest == ">-")
                    {
                        var val = new StringBuilder();
                        i++;
                        while (i < n && (lines[i].StartsWith(" ") || lines[i].StartsWith("\t")))
                        {
                            if (val.Length > 0) val.Append('\n');
                            val.Append(TrimLeadingSpaceTab(lines[i]));
                            i++;
                        }
                        f(key, val.ToString());
                        continue;
                    }
                    string v = rest;
                    if (v.StartsWith("\"") && v.EndsWith("\"") && v.Length >= 2)
                    {
                        v = v.Substring(1, v.Length - 2);
                    }
                    f(key, v);
                }
                i++;
            }
        }

        private static Dictionary<string, string> FmMap(List<string> lines)
        {
            var map = new Dictionary<string, string>();
            ForEachFmEntry(lines, (k, v) => map[k] = v);
            return map;
        }

        private static string ExtractReservedField(List<string> lines, string field)
        {
            int i = 0;
            int n = lines.Count;
            while (i < n)
            {
                string trimmed = lines[i].Trim();
                int colon = trimmed.IndexOf(':');
                if (colon >= 0)
                {
                    string k = trimmed.Substring(0, colon).Trim();
                    if (k == field)
                    {
                        string rest = trimmed.Substring(colon + 1).Trim();
                        if (rest == "|" || rest == ">" || rest == "|-" || rest == ">-")
                        {
                            var val = new StringBuilder();
                            i++;
                            while (i < n)
                            {
                                string L = lines[i];
                                bool empty = L.Trim().Length == 0;
                                bool indented = L.StartsWith(" ") || L.StartsWith("\t");
                                if (indented || (empty && i + 1 < n && (lines[i + 1].StartsWith(" ") || lines[i + 1].StartsWith("\t"))))
                                {
                                    if (empty)
                                    {
                                        val.Append('\n');
                                        i++;
                                        continue;
                                    }
                                    if (val.Length > 0) val.Append('\n');
                                    val.Append(TrimLeadingSpaceTab(L));
                                    i++;
                                    continue;
                                }
                                break;
                            }
                            return val.ToString();
                        }
                        if (rest.StartsWith("\"") && rest.EndsWith("\"") && rest.Length >= 2)
                        {
                            rest = rest.Substring(1, rest.Length - 2);
                        }
                        return rest;
                    }
                }
                i++;
            }
            return null;
        }

        private static string CanonicalFmString(Dictionary<string, string> map)
        {
            var keys = new List<string>(map.Keys);
            keys.Sort(StringComparer.Ordinal);
            var s = new StringBuilder();
            foreach (string k in keys)
            {
                string v = map[k];
                s.Append(k).Append(": ");
                if (v.Length == 0 || v.IndexOf(':') >= 0 || v.IndexOf('#') >= 0 || v.IndexOf(' ') >= 0)
                {
                    s.Append('"').Append(v.Replace("\"", "\\\"")).Append('"');
                }
                else
                {
                    s.Append(v);
                }
                s.Append('\n');
            }
            return s.ToString();
        }

        private static byte[] HashPayload(ParsedDoc doc)
        {
            string bodyLf = NormalizeLf(doc.BodyRaw);
            var map = FmMap(doc.FmLines);
            if (map.Count == 0)
            {
                return Encoding.UTF8.GetBytes(bodyLf);
            }
            string payload = CanonicalFmString(map) + "\n" + bodyLf;
            return Encoding.UTF8.GetBytes(payload);
        }

        private static Digest ComputeDigest(ParsedDoc doc)
        {
            return Blake3Digest(HashPayload(doc));
        }

        private static Digest ParseDigest(string raw)
        {
            string s = raw.Trim();
            if (s.StartsWith("\"") && s.EndsWith("\"") && s.Length >= 2)
            {
                s = s.Substring(1, s.Length - 2);
            }
            int idx = s.IndexOf(':');
            if (idx < 0) return null;
            string algorithm = s.Substring(0, idx).ToLowerInvariant();
            string hex = s.Substring(idx + 1).Trim().ToLowerInvariant();
            if (hex.Length == 0) return null;
            for (int i = 0; i < hex.Length; i++)
            {
                char c = hex[i];
                if (!((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'))) return null;
            }
            return new Digest { Algorithm = algorithm, Hex = hex, Qualified = algorithm + ":" + hex };
        }
    }
}
