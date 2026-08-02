# frozen_string_literal: true
require "json"
require_relative "lib/hashseal_verify"

vec = File.expand_path("../vectors/instruct-v1.json", __dir__)
doc = JSON.parse(File.read(vec))
failed = 0
doc["cases"].each do |c|
  next if c["text"].nil? || c["text"].empty?

  r = HashsealVerify.check_document_text(c["text"])
  exp = c["expect"]
  ok = r["ok"] == exp["ok"] && r["status"] == exp["status"]
  unless ok
    warn "FAIL #{c['id']}: got #{r.inspect} want #{exp.inspect}"
    failed += 1
  end
end
abort "failed=#{failed}" if failed.positive?
puts "ruby vectors: all ok"
