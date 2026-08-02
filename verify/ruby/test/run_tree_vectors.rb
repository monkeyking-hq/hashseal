# frozen_string_literal: true

# Run: ruby test/run_tree_vectors.rb
# Copyright (c) 2026 MonkeyKing.dev

require "json"
require_relative "../lib/hashseal_tree"

vec_path = File.expand_path("../../vectors/tree-v1.json", __dir__)
doc = JSON.parse(File.read(vec_path))
abort "unexpected spec #{doc['spec']}" unless doc["spec"] == "tree-v1"

lf_text = doc["line_endings_lf_text"] != false
text_exts = doc["text_extensions"]
passed = 0
failed = 0

doc["cases"].each do |c|
  begin
    if c["kind"] == "raw_file_digest"
      r = Hashseal.hash_tree_file_content(
        c["path"], c["content"],
        line_endings_lf_text: lf_text,
        text_extensions: text_exts
      )
      raise "digest: got #{r[:digest]} want #{c['expect']['digest']}" if r[:digest] != c["expect"]["digest"]
      raise "size: got #{r[:size]} want #{c['expect']['size']}" if r[:size] != c["expect"]["size"]
    elsif c["kind"] == "verify_tree"
      r = Hashseal.verify_tree_in_memory(
        c["files"] || {},
        c["ledger_entries"] || [],
        line_endings_lf_text: lf_text,
        text_extensions: text_exts
      )
      exp = c["expect"]
      raise "ok: got #{r['ok']} want #{exp['ok']}" if r["ok"] != exp["ok"]
      raise "checked: got #{r['checked']} want #{exp['checked']}" if r["checked"] != exp["checked"]
      want = exp["findings"] || []
      raise "findings.length: got #{r['findings'].length} want #{want.length}" if r["findings"].length != want.length

      want.each_with_index do |w, i|
        g = r["findings"][i]
        %w[path status expected actual].each do |k|
          gv = g[k]
          wv = w[k]
          raise "finding[#{i}].#{k}: got #{gv.inspect} want #{wv.inspect}" if gv != wv
        end
      end
    else
      raise "unknown kind #{c['kind']}"
    end
    passed += 1
    puts "ok  #{c['id']}"
  rescue StandardError => e
    failed += 1
    warn "FAIL #{c['id']}: #{e.message}"
  end
end

puts "\n#{passed} passed, #{failed} failed"
exit(failed.zero? ? 0 : 1)
