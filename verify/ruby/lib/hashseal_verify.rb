# frozen_string_literal: true
# HashSeal Ruby verify — zero gems.
# Digest parity via monorepo Python pure-BLAKE3 check (stdlib subprocess).
#
# Copyright (c) 2026 MonkeyKing.dev

require "json"
require "open3"

module HashsealVerify
  module_function

  # Returns Hash with string keys: ok, status, algorithm, expected, actual, message
  def check_document_text(text, field: "hashseal")
    py_root = File.expand_path("../../python", __dir__)
    code = <<~PY
      import json, sys
      sys.path.insert(0, #{py_root.dump})
      from check import check_document_text
      print(json.dumps(check_document_text(sys.stdin.read(), field=#{field.dump})))
    PY
    out, err, st = Open3.capture3("python", "-c", code, stdin_data: text)
    unless st.success?
      return {
        "ok" => false,
        "status" => "invalid_format",
        "algorithm" => nil,
        "expected" => nil,
        "actual" => nil,
        "message" => "python check failed: #{err}"
      }
    end
    JSON.parse(out)
  end
end
