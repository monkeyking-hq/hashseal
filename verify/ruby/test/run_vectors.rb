#!/usr/bin/env ruby
# frozen_string_literal: true

# Run official instruct-v1 vectors.
# Usage: ruby test/run_vectors.rb
#
# Copyright (c) 2026 MonkeyKing.dev

require "json"
require "pathname"

ROOT = Pathname.new(__dir__).parent
require ROOT.join("lib/hashseal_check")

VECTORS = ROOT.parent.join("vectors/instruct-v1.json")

doc = JSON.parse(VECTORS.read)
abort "unexpected spec #{doc['spec']}" if doc["spec"] != "instruct-v1"

passed = 0
failed = 0

def assert_eq(a, b, label)
  raise "#{label}: got #{a.inspect} want #{b.inspect}" if a != b
end

doc["cases"].each do |c|
  begin
    if c["kind"] == "raw_digest"
      actual = Hashseal.blake3_digest(c["bytes_utf8"])[:qualified]
      assert_eq(actual, c["expect"]["digest"], "#{c['id']} digest")
    elsif c["kind"] == "check"
      r = Hashseal.check_document_text(c["text"])
      assert_eq(r[:ok], c["expect"]["ok"], "#{c['id']} ok")
      assert_eq(r[:status], c["expect"]["status"], "#{c['id']} status")
      if !c["expect"]["digest"].nil?
        assert_eq(r[:actual], c["expect"]["digest"], "#{c['id']} actual digest")
        assert_eq(r[:expected], c["expect"]["digest"], "#{c['id']} expected digest") if r[:ok]
      end
      assert_eq(r[:expected], c["expect"]["expected"], "#{c['id']} expected") if c["expect"].key?("expected") && !c["expect"]["expected"].nil?
      assert_eq(r[:actual], c["expect"]["actual"], "#{c['id']} actual") if c["expect"].key?("actual") && !c["expect"]["actual"].nil?
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
