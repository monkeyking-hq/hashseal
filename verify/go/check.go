// Package hashseal — instruct document check (FULL canonical mode).
// Mirrors hashseal-core instruct algorithm. Zero external package deps.
//
// Copyright (c) 2026 MonkeyKing.dev
package hashseal

import (
	"sort"
	"strings"
)

// Seal field names.
const (
	SealField  = "hashseal"
	SigField   = "hashseal_sig"
	KeyIDField = "hashseal_key_id"
)

// Result of CheckDocumentText.
type Result struct {
	OK        bool    `json:"ok"`
	Status    string  `json:"status"` // valid | mismatch | missing_seal | invalid_format
	Algorithm *string `json:"algorithm"`
	Expected  *string `json:"expected"`
	Actual    *string `json:"actual"`
	Message   *string `json:"message"`
}

// Digest is a parsed or computed digest.
type Digest struct {
	Algorithm string
	Hex       string
	Qualified string
}

func strPtr(s string) *string { return &s }

// CheckDocumentText checks a sealed instruct markdown document (FULL mode).
func CheckDocumentText(text string) Result {
	return CheckDocumentTextField(text, SealField)
}

// CheckDocumentTextField is like CheckDocumentText with a custom seal field name.
func CheckDocumentTextField(text, field string) Result {
	doc := parseDocument(text)
	if !doc.hadFrontMatter {
		actual := computeDigest(doc)
		return Result{
			OK: false, Status: "missing_seal", Algorithm: strPtr("blake3"),
			Expected: nil, Actual: strPtr(actual.Qualified), Message: strPtr("missing hashseal field"),
		}
	}
	sealRaw := extractReservedField(doc.fmLines, field)
	if sealRaw == nil {
		actual := computeDigest(doc)
		return Result{
			OK: false, Status: "missing_seal", Algorithm: strPtr("blake3"),
			Expected: nil, Actual: strPtr(actual.Qualified), Message: strPtr("missing hashseal field"),
		}
	}
	expected := parseDigest(*sealRaw)
	if expected == nil {
		return Result{
			OK: false, Status: "invalid_format",
			Message: strPtr("invalid digest format: " + *sealRaw),
		}
	}
	if expected.Algorithm != "blake3" {
		return Result{
			OK: false, Status: "invalid_format",
			Algorithm: strPtr(expected.Algorithm), Expected: strPtr(expected.Qualified),
			Message: strPtr("unsupported algorithm: " + expected.Algorithm),
		}
	}
	actual := computeDigest(doc)
	if actual.Hex != expected.Hex || actual.Algorithm != expected.Algorithm {
		return Result{
			OK: false, Status: "mismatch", Algorithm: strPtr(expected.Algorithm),
			Expected: strPtr(expected.Qualified), Actual: strPtr(actual.Qualified),
		}
	}
	return Result{
		OK: true, Status: "valid", Algorithm: strPtr(actual.Algorithm),
		Expected: strPtr(expected.Qualified), Actual: strPtr(actual.Qualified),
	}
}

// Blake3Hex returns lowercase hex of the 32-byte BLAKE3 hash.
func Blake3Hex(data []byte) string {
	sum := Sum256(data)
	const digits = "0123456789abcdef"
	out := make([]byte, 64)
	for i, b := range sum {
		out[i*2] = digits[b>>4]
		out[i*2+1] = digits[b&0x0f]
	}
	return string(out)
}

// Blake3Digest returns algorithm/hex/qualified for data.
func Blake3Digest(data []byte) Digest {
	hex := Blake3Hex(data)
	return Digest{Algorithm: "blake3", Hex: hex, Qualified: "blake3:" + hex}
}

// Blake3DigestString hashes UTF-8 text.
func Blake3DigestString(s string) Digest {
	return Blake3Digest([]byte(s))
}

type parsedDoc struct {
	fmLines        []string
	hadFrontMatter bool
	bodyRaw        string
}

func stripBom(s string) string {
	if strings.HasPrefix(s, "\ufeff") {
		return strings.TrimPrefix(s, "\ufeff")
	}
	return s
}

func normalizeLf(s string) string {
	s = strings.ReplaceAll(s, "\r\n", "\n")
	s = strings.ReplaceAll(s, "\r", "\n")
	return s
}

func parseDocument(text string) parsedDoc {
	text = stripBom(text)
	if strings.HasPrefix(text, "---\n") || strings.HasPrefix(text, "---\r\n") {
		var afterOpen string
		if strings.HasPrefix(text, "---\r\n") {
			afterOpen = text[5:]
		} else {
			afterOpen = text[4:]
		}
		search := afterOpen
		offset := 0
		for {
			idx := strings.Index(search, "\n---")
			if idx < 0 {
				break
			}
			after := search[idx+1:]
			rest := after[3:]
			closed := len(rest) == 0 || strings.HasPrefix(rest, "\n") || strings.HasPrefix(rest, "\r\n") || strings.HasPrefix(rest, "\r")
			if closed {
				fmBlock := afterOpen[:offset+idx]
				body := afterOpen[idx+1+3:]
				if strings.HasPrefix(body, "\r\n") {
					body = body[2:]
				} else if strings.HasPrefix(body, "\n") {
					body = body[1:]
				} else if strings.HasPrefix(body, "\r") {
					body = body[1:]
				}
				fmLines := strings.Split(normalizeLf(fmBlock), "\n")
				return parsedDoc{fmLines: fmLines, hadFrontMatter: true, bodyRaw: body}
			}
			offset += idx + 1
			search = search[idx+1:]
		}
	}
	return parsedDoc{fmLines: nil, hadFrontMatter: false, bodyRaw: text}
}

func isReservedKey(key string) bool {
	return key == SealField || key == SigField || key == KeyIDField
}

func trimLeadingSpaceTab(s string) string {
	i := 0
	for i < len(s) && (s[i] == ' ' || s[i] == '\t') {
		i++
	}
	return s[i:]
}

func forEachFmEntry(lines []string, f func(key, val string)) {
	i := 0
	n := len(lines)
	for i < n {
		line := lines[i]
		trimmed := strings.TrimSpace(line)
		if trimmed == "" || strings.HasPrefix(trimmed, "#") {
			i++
			continue
		}
		if strings.HasPrefix(line, " ") || strings.HasPrefix(line, "\t") {
			i++
			continue
		}
		colon := strings.Index(trimmed, ":")
		if colon >= 0 {
			key := strings.TrimSpace(trimmed[:colon])
			rest := strings.TrimSpace(trimmed[colon+1:])
			if isReservedKey(key) {
				i++
				for i < n {
					L := lines[i]
					if strings.HasPrefix(L, " ") || strings.HasPrefix(L, "\t") {
						i++
						continue
					}
					if strings.TrimSpace(L) == "" {
						if i+1 < n && (strings.HasPrefix(lines[i+1], " ") || strings.HasPrefix(lines[i+1], "\t")) {
							i++
							continue
						}
						break
					}
					break
				}
				continue
			}
			if rest == "|" || rest == ">" || rest == "|-" || rest == ">-" {
				val := ""
				i++
				for i < n && (strings.HasPrefix(lines[i], " ") || strings.HasPrefix(lines[i], "\t")) {
					if val != "" {
						val += "\n"
					}
					val += trimLeadingSpaceTab(lines[i])
					i++
				}
				f(key, val)
				continue
			}
			val := rest
			if len(val) >= 2 && strings.HasPrefix(val, "\"") && strings.HasSuffix(val, "\"") {
				val = val[1 : len(val)-1]
			}
			f(key, val)
		}
		i++
	}
}

func fmMap(lines []string) map[string]string {
	m := make(map[string]string)
	forEachFmEntry(lines, func(k, v string) {
		m[k] = v
	})
	return m
}

func extractReservedField(lines []string, field string) *string {
	i := 0
	n := len(lines)
	for i < n {
		trimmed := strings.TrimSpace(lines[i])
		colon := strings.Index(trimmed, ":")
		if colon >= 0 {
			k := strings.TrimSpace(trimmed[:colon])
			if k == field {
				rest := strings.TrimSpace(trimmed[colon+1:])
				if rest == "|" || rest == ">" || rest == "|-" || rest == ">-" {
					val := ""
					i++
					for i < n {
						L := lines[i]
						empty := strings.TrimSpace(L) == ""
						indented := strings.HasPrefix(L, " ") || strings.HasPrefix(L, "\t")
						if indented || (empty && i+1 < n && (strings.HasPrefix(lines[i+1], " ") || strings.HasPrefix(lines[i+1], "\t"))) {
							if empty {
								val += "\n"
								i++
								continue
							}
							if val != "" {
								val += "\n"
							}
							val += trimLeadingSpaceTab(L)
							i++
							continue
						}
						break
					}
					return &val
				}
				if len(rest) >= 2 && strings.HasPrefix(rest, "\"") && strings.HasSuffix(rest, "\"") {
					rest = rest[1 : len(rest)-1]
				}
				return &rest
			}
		}
		i++
	}
	return nil
}

func canonicalFmString(m map[string]string) string {
	keys := make([]string, 0, len(m))
	for k := range m {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	var b strings.Builder
	for _, k := range keys {
		v := m[k]
		b.WriteString(k)
		b.WriteString(": ")
		if v == "" || strings.Contains(v, ":") || strings.Contains(v, "#") || strings.Contains(v, " ") {
			b.WriteByte('"')
			b.WriteString(strings.ReplaceAll(v, `"`, `\"`))
			b.WriteByte('"')
		} else {
			b.WriteString(v)
		}
		b.WriteByte('\n')
	}
	return b.String()
}

func hashPayload(doc parsedDoc) []byte {
	bodyLf := normalizeLf(doc.bodyRaw)
	m := fmMap(doc.fmLines)
	if len(m) == 0 {
		return []byte(bodyLf)
	}
	payload := canonicalFmString(m) + "\n" + bodyLf
	return []byte(payload)
}

func computeDigest(doc parsedDoc) Digest {
	return Blake3Digest(hashPayload(doc))
}

func parseDigest(raw string) *Digest {
	s := strings.TrimSpace(raw)
	if len(s) >= 2 && strings.HasPrefix(s, "\"") && strings.HasSuffix(s, "\"") {
		s = s[1 : len(s)-1]
	}
	idx := strings.Index(s, ":")
	if idx < 0 {
		return nil
	}
	algorithm := strings.ToLower(s[:idx])
	hex := strings.ToLower(strings.TrimSpace(s[idx+1:]))
	if hex == "" {
		return nil
	}
	for _, c := range hex {
		if !((c >= '0' && c <= '9') || (c >= 'a' && c <= 'f')) {
			return nil
		}
	}
	return &Digest{Algorithm: algorithm, Hex: hex, Qualified: algorithm + ":" + hex}
}
