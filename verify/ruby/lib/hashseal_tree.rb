# frozen_string_literal: true

# In-memory tree verify — mirrors hashseal-core / verify/js tree.js.
# Zero gem dependencies.
#
# Copyright (c) 2026 MonkeyKing.dev

require_relative "../vendor/blake3"

module Hashseal
  DEFAULT_TEXT_EXTENSIONS = %w[
    md txt toml yml yaml json rs java go py js ts tsx jsx css html xml
    sh ps1 c h cpp cs rb svg
  ].freeze

  module_function

  def normalize_lf(s)
    s.to_s.gsub("\r\n", "\n").gsub("\r", "\n")
  end

  def ext_of(path)
    i = path.rindex(".")
    return "" if i.nil?

    path[(i + 1)..].to_s.downcase
  end

  # Hash one path+content with core tree policy.
  # size is UTF-8 byte length before normalize.
  def hash_tree_file_content(path, content, line_endings_lf_text: true, text_extensions: nil)
    text_exts = text_extensions ? text_extensions.map(&:to_s) : DEFAULT_TEXT_EXTENSIONS
    content = content.to_s
    size = content.bytesize
    data = content
    if line_endings_lf_text && text_exts.include?(ext_of(path))
      data = data.sub(/\A\uFEFF/, "")
      data = normalize_lf(data)
    end
    hex = Blake3.hexdigest(data)
    qualified = "blake3:#{hex}"
    { digest: qualified, qualified: qualified, hex: hex, size: size }
  end

  # Verify in-memory files against ledger entries.
  # files: Hash path => content
  # ledger_entries: Array of hashes with "path", "digest"
  def verify_tree_in_memory(files, ledger_entries, line_endings_lf_text: true, text_extensions: nil)
    files = files || {}
    current = {}
    files.keys.sort.each do |p|
      h = hash_tree_file_content(
        p, files[p],
        line_endings_lf_text: line_endings_lf_text,
        text_extensions: text_extensions
      )
      current[p] = h[:qualified]
    end

    findings = []
    expected_paths = {}
    (ledger_entries || []).each do |e|
      path = e["path"] || e[:path]
      digest = e["digest"] || e[:digest]
      expected_paths[path] = true
      actual = current[path]
      if actual.nil?
        findings << {
          "path" => path,
          "status" => "removed",
          "expected" => digest,
          "actual" => nil
        }
      elsif actual != digest
        findings << {
          "path" => path,
          "status" => "mismatch",
          "expected" => digest,
          "actual" => actual
        }
      end
    end

    current.keys.sort.each do |path|
      next if expected_paths[path]

      findings << {
        "path" => path,
        "status" => "added",
        "expected" => nil,
        "actual" => current[path]
      }
    end

    findings.sort_by! { |f| f["path"] }
    {
      "ok" => findings.empty?,
      "checked" => (ledger_entries || []).length,
      "findings" => findings
    }
  end
end
