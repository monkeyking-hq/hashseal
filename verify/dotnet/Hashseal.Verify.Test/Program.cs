// Run official instruct-v1 and tree-v1 vectors.
// Usage:
//   dotnet run --project Hashseal.Verify.Test
//   dotnet run --project Hashseal.Verify.Test -- tree
//
// Copyright (c) 2026 MonkeyKing.dev

using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using Hashseal.Verify;

static class Program
{
    static int Main(string[] args)
    {
        bool treeOnly = args.Any(a => a.Equals("tree", StringComparison.OrdinalIgnoreCase));
        bool instructOnly = args.Any(a => a.Equals("instruct", StringComparison.OrdinalIgnoreCase));
        int failed = 0;
        if (!treeOnly)
            failed += RunInstruct();
        if (!instructOnly)
            failed += RunTree();
        return failed == 0 ? 0 : 1;
    }

    static int RunInstruct()
    {
        string vectorsPath = FindVectors("instruct-v1.json");
        if (vectorsPath == null)
        {
            Console.Error.WriteLine("could not find instruct-v1.json");
            return 1;
        }

        string json = File.ReadAllText(vectorsPath);
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        if (root.GetProperty("spec").GetString() != "instruct-v1")
        {
            Console.Error.WriteLine("unexpected instruct spec");
            return 1;
        }

        int passed = 0, failed = 0;
        foreach (var c in root.GetProperty("cases").EnumerateArray())
        {
            string id = c.GetProperty("id").GetString() ?? "?";
            try
            {
                string kind = c.GetProperty("kind").GetString() ?? "";
                if (kind == "raw_digest")
                {
                    string bytes = c.GetProperty("bytes_utf8").GetString() ?? "";
                    string actual = Check.Blake3Digest(bytes).Qualified;
                    string want = c.GetProperty("expect").GetProperty("digest").GetString() ?? "";
                    AssertEq(actual, want, id + " digest");
                }
                else if (kind == "check")
                {
                    string text = c.GetProperty("text").GetString() ?? "";
                    var r = Check.CheckDocumentText(text);
                    var expect = c.GetProperty("expect");
                    AssertEq(r.Ok, expect.GetProperty("ok").GetBoolean(), id + " ok");
                    AssertEq(r.Status, expect.GetProperty("status").GetString(), id + " status");
                    if (expect.TryGetProperty("digest", out var dig) && dig.ValueKind == JsonValueKind.String)
                    {
                        AssertEq(r.Actual, dig.GetString(), id + " actual digest");
                        if (r.Ok)
                            AssertEq(r.Expected, dig.GetString(), id + " expected digest");
                    }
                }
                else
                {
                    throw new Exception("unknown kind " + kind);
                }
                passed++;
                Console.WriteLine("ok  " + id);
            }
            catch (Exception e)
            {
                failed++;
                Console.Error.WriteLine("FAIL " + id + ": " + e.Message);
            }
        }
        Console.WriteLine($"instruct: {passed} passed, {failed} failed");
        return failed;
    }

    static int RunTree()
    {
        string vectorsPath = FindVectors("tree-v1.json");
        if (vectorsPath == null)
        {
            Console.Error.WriteLine("could not find tree-v1.json");
            return 1;
        }

        string json = File.ReadAllText(vectorsPath);
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        if (root.GetProperty("spec").GetString() != "tree-v1")
        {
            Console.Error.WriteLine("unexpected tree spec");
            return 1;
        }

        bool lfText = !root.TryGetProperty("line_endings_lf_text", out var lfEl) || lfEl.GetBoolean();
        HashSet<string>? textExts = null;
        if (root.TryGetProperty("text_extensions", out var te) && te.ValueKind == JsonValueKind.Array)
        {
            textExts = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (var x in te.EnumerateArray())
                textExts.Add(x.GetString() ?? "");
        }

        int passed = 0, failed = 0;
        foreach (var c in root.GetProperty("cases").EnumerateArray())
        {
            string id = c.GetProperty("id").GetString() ?? "?";
            try
            {
                string kind = c.GetProperty("kind").GetString() ?? "";
                if (kind == "raw_file_digest")
                {
                    string path = c.GetProperty("path").GetString() ?? "";
                    string content = c.GetProperty("content").GetString() ?? "";
                    var r = Tree.HashTreeFileContent(path, content, lfText, textExts);
                    var expect = c.GetProperty("expect");
                    AssertEq(r.Digest, expect.GetProperty("digest").GetString(), id + " digest");
                    AssertEq(r.Size, expect.GetProperty("size").GetInt32(), id + " size");
                }
                else if (kind == "verify_tree")
                {
                    var files = new Dictionary<string, string>();
                    if (c.TryGetProperty("files", out var filesEl) && filesEl.ValueKind == JsonValueKind.Object)
                    {
                        foreach (var p in filesEl.EnumerateObject())
                            files[p.Name] = p.Value.GetString() ?? "";
                    }
                    var entries = new List<LedgerEntryLike>();
                    if (c.TryGetProperty("ledger_entries", out var le) && le.ValueKind == JsonValueKind.Array)
                    {
                        foreach (var e in le.EnumerateArray())
                        {
                            entries.Add(new LedgerEntryLike
                            {
                                Path = e.GetProperty("path").GetString() ?? "",
                                Digest = e.GetProperty("digest").GetString() ?? ""
                            });
                        }
                    }
                    var r = Tree.VerifyTreeInMemory(files, entries, lfText, textExts);
                    var expect = c.GetProperty("expect");
                    AssertEq(r.Ok, expect.GetProperty("ok").GetBoolean(), id + " ok");
                    AssertEq(r.Checked, expect.GetProperty("checked").GetInt32(), id + " checked");
                    var want = expect.GetProperty("findings");
                    AssertEq(r.Findings.Count, want.GetArrayLength(), id + " findings.length");
                    int i = 0;
                    foreach (var w in want.EnumerateArray())
                    {
                        var g = r.Findings[i];
                        AssertEq(g.Path, w.GetProperty("path").GetString(), id + $" finding[{i}].path");
                        AssertEq(g.Status, w.GetProperty("status").GetString(), id + $" finding[{i}].status");
                        AssertEq(g.Expected, JsonNullString(w, "expected"), id + $" finding[{i}].expected");
                        AssertEq(g.Actual, JsonNullString(w, "actual"), id + $" finding[{i}].actual");
                        i++;
                    }
                }
                else
                {
                    throw new Exception("unknown kind " + kind);
                }
                passed++;
                Console.WriteLine("ok  " + id);
            }
            catch (Exception e)
            {
                failed++;
                Console.Error.WriteLine("FAIL " + id + ": " + e.Message);
            }
        }
        Console.WriteLine($"tree: {passed} passed, {failed} failed");
        return failed;
    }

    static string? JsonNullString(JsonElement parent, string name)
    {
        if (!parent.TryGetProperty(name, out var el) || el.ValueKind == JsonValueKind.Null)
            return null;
        return el.GetString();
    }

    static void AssertEq(object? a, object? b, string label)
    {
        if (!Equals(a, b))
            throw new Exception(label + ": got " + (a ?? "null") + " want " + (b ?? "null"));
    }

    static string? FindVectors(string fileName)
    {
        var dir = new DirectoryInfo(AppContext.BaseDirectory);
        for (int i = 0; i < 10 && dir != null; i++)
        {
            string candidate = Path.Combine(dir.FullName, "vectors", fileName);
            if (File.Exists(candidate)) return candidate;
            candidate = Path.GetFullPath(Path.Combine(dir.FullName, "..", "vectors", fileName));
            if (File.Exists(candidate)) return candidate;
            dir = dir.Parent;
        }
        string fromCwd = Path.GetFullPath(Path.Combine(Directory.GetCurrentDirectory(), "..", "vectors", fileName));
        if (File.Exists(fromCwd)) return fromCwd;
        fromCwd = Path.GetFullPath(Path.Combine(Directory.GetCurrentDirectory(), "verify", "vectors", fileName));
        if (File.Exists(fromCwd)) return fromCwd;
        return null;
    }
}
