# frozen_string_literal: true

# HashSeal instruct document check — FULL canonical mode (digest only).
# Mirrors hashseal-core instruct algorithm. Zero gem dependencies.
#
# Copyright (c) 2026 MonkeyKing.dev

require_relative "../vendor/blake3"

module Hashseal
  SEAL_FIELD = "hashseal"
  SIG_FIELD = "hashseal_sig"
  KEY_ID_FIELD = "hashseal_key_id"
  RESERVED = [SEAL_FIELD, SIG_FIELD, KEY_ID_FIELD].freeze

  module_function

  def check_document_text(text, field: SEAL_FIELD)
    doc = parse_document(text)
    unless doc[:had_front_matter]
      actual = compute_digest(doc)
      return result(false, "missing_seal", "blake3", nil, actual[:qualified], "missing hashseal field")
    end
    seal_raw = extract_reserved_field(doc[:fm_lines], field)
    if seal_raw.nil?
      actual = compute_digest(doc)
      return result(false, "missing_seal", "blake3", nil, actual[:qualified], "missing hashseal field")
    end
    expected = parse_digest(seal_raw)
    if expected.nil?
      return result(false, "invalid_format", nil, nil, nil, "invalid digest format: #{seal_raw}")
    end
    if expected[:algorithm] != "blake3"
      return result(
        false, "invalid_format", expected[:algorithm], expected[:qualified], nil,
        "unsupported algorithm: #{expected[:algorithm]}"
      )
    end
    actual = compute_digest(doc)
    if actual[:hex] != expected[:hex] || actual[:algorithm] != expected[:algorithm]
      return result(false, "mismatch", expected[:algorithm], expected[:qualified], actual[:qualified], nil)
    end
    result(true, "valid", actual[:algorithm], expected[:qualified], actual[:qualified], nil)
  end

  def blake3_digest(data)
    bytes =
      if data.is_a?(String)
        data.encoding == Encoding::ASCII_8BIT ? data : data.encode("UTF-8").b
      else
        data
      end
    hex = Blake3.hexdigest(bytes)
    { algorithm: "blake3", hex: hex, qualified: "blake3:#{hex}" }
  end

  def result(ok, status, algorithm, expected, actual, message)
    {
      ok: ok,
      status: status,
      algorithm: algorithm,
      expected: expected,
      actual: actual,
      message: message
    }
  end

  def strip_bom(s)
    s.start_with?("\uFEFF") ? s[1..] : s
  end

  def normalize_lf(s)
    s.gsub("\r\n", "\n").gsub("\r", "\n")
  end

  def parse_document(text)
    text = strip_bom(text)
    if text.start_with?("---\n") || text.start_with?("---\r\n")
      after_open = text.start_with?("---\r\n") ? text[5..] : text[4..]
      search = after_open
      offset = 0
      loop do
        idx = search.index("\n---")
        break if idx.nil?

        after = search[(idx + 1)..]
        rest = after[3..] || ""
        closed = rest.empty? || rest.start_with?("\n") || rest.start_with?("\r\n") || rest.start_with?("\r")
        if closed
          fm_block = after_open[0, offset + idx]
          body = after_open[(idx + 1 + 3)..] || ""
          if body.start_with?("\r\n")
            body = body[2..]
          elsif body.start_with?("\n")
            body = body[1..]
          elsif body.start_with?("\r")
            body = body[1..]
          end
          fm_lines = normalize_lf(fm_block).split("\n", -1)
          return { fm_lines: fm_lines, had_front_matter: true, body_raw: body }
        end
        offset += idx + 1
        search = search[(idx + 1)..]
      end
    end
    { fm_lines: [], had_front_matter: false, body_raw: text }
  end

  def reserved_key?(key)
    RESERVED.include?(key)
  end

  def for_each_fm_entry_clean(lines)
    i = 0
    n = lines.length
    while i < n
      line = lines[i]
      trimmed = line.strip
      if trimmed.empty? || trimmed.start_with?("#")
        i += 1
        next
      end
      if line.start_with?(" ") || line.start_with?("\t")
        i += 1
        next
      end
      colon = trimmed.index(":")
      if colon
        key = trimmed[0...colon].strip
        rest = trimmed[(colon + 1)..].to_s.strip
        if reserved_key?(key)
          i += 1
          while i < n
            l = lines[i]
            if l.start_with?(" ") || l.start_with?("\t")
              i += 1
              next
            end
            if l.strip.empty?
              if i + 1 < n && (lines[i + 1].start_with?(" ") || lines[i + 1].start_with?("\t"))
                i += 1
                next
              end
              break
            end
            break
          end
          next
        end
        if ["|", ">", "|-", ">-"].include?(rest)
          val = +""
          i += 1
          while i < n && (lines[i].start_with?(" ") || lines[i].start_with?("\t"))
            val << "\n" unless val.empty?
            l = lines[i]
            j = 0
            j += 1 while j < l.length && (l[j] == " " || l[j] == "\t")
            val << l[j..]
            i += 1
          end
          yield key, val
          next
        end
        val = rest
        if val.start_with?("\"") && val.end_with?("\"") && val.length >= 2
          val = val[1...-1]
        end
        yield key, val
      end
      i += 1
    end
  end

  def fm_map(lines)
    m = {}
    for_each_fm_entry_clean(lines) { |k, v| m[k] = v }
    m
  end

  def extract_reserved_field(lines, field)
    i = 0
    n = lines.length
    while i < n
      trimmed = lines[i].strip
      colon = trimmed.index(":")
      if colon
        k = trimmed[0...colon].strip
        if k == field
          rest = trimmed[(colon + 1)..].to_s.strip
          if ["|", ">", "|-", ">-"].include?(rest)
            val = +""
            i += 1
            while i < n
              l = lines[i]
              empty = l.strip.empty?
              indented = l.start_with?(" ") || l.start_with?("\t")
              if indented || (empty && i + 1 < n && (lines[i + 1].start_with?(" ") || lines[i + 1].start_with?("\t")))
                if empty
                  val << "\n"
                  i += 1
                  next
                end
                val << "\n" unless val.empty?
                j = 0
                j += 1 while j < l.length && (l[j] == " " || l[j] == "\t")
                val << l[j..]
                i += 1
                next
              end
              break
            end
            return val
          end
          if rest.start_with?("\"") && rest.end_with?("\"") && rest.length >= 2
            rest = rest[1...-1]
          end
          return rest
        end
      end
      i += 1
    end
    nil
  end

  def canonical_fm_string(m)
    keys = m.keys.sort
    parts = +""
    keys.each do |k|
      v = m[k]
      parts << k << ": "
      if v.empty? || v.include?(":") || v.include?("#") || v.include?(" ")
        parts << "\"" << v.gsub("\"", "\\\"") << "\""
      else
        parts << v
      end
      parts << "\n"
    end
    parts
  end

  def hash_payload(doc)
    body_lf = normalize_lf(doc[:body_raw])
    m = fm_map(doc[:fm_lines])
    return body_lf.b if m.empty?

    (canonical_fm_string(m) + "\n" + body_lf).b
  end

  def compute_digest(doc)
    blake3_digest(hash_payload(doc))
  end

  def parse_digest(raw)
    s = raw.to_s.strip
    s = s[1...-1] if s.start_with?("\"") && s.end_with?("\"") && s.length >= 2
    idx = s.index(":")
    return nil if idx.nil?

    algorithm = s[0...idx].downcase
    hex = s[(idx + 1)..].to_s.strip.downcase
    return nil if hex.empty? || hex !~ /\A[0-9a-f]+\z/

    { algorithm: algorithm, hex: hex, qualified: "#{algorithm}:#{hex}" }
  end
end
