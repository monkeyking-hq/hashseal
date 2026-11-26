// Package hashseal — instruct document check (instruct-v2: lossless raw-FM + Unicode + SSHSIG).
// Mirrors hashseal-core instruct.rs & unicode.rs. Zero external package deps.
//
// Copyright (c) 2026 MonkeyKing.dev
package hashseal

import (
	"fmt"
	"strings"
)

// Seal field names.
const (
	SealField      = "hashseal"
	SigField       = "hashseal_sig"
	SigFormatField = "hashseal_sig_format"
	SignerField    = "hashseal_signer"
	ScopeField     = "hashseal_scope"
	KeyIDField     = "hashseal_key_id"
)

const instructV2Domain = "HASHSEAL-INSTRUCT-V2"

// Result of CheckDocumentText.
type Result struct {
	OK        bool    `json:"ok"`
	Status    string  `json:"status"`
	Algorithm *string `json:"algorithm"`
	Expected  *string `json:"expected"`
	Actual    *string `json:"actual"`
	Signer    *string `json:"signer,omitempty"`
	Message   *string `json:"message"`
}

// Digest is a parsed or computed digest.
type Digest struct {
	Algorithm string
	Hex       string
	Qualified string
}

// CheckOptions configures signature and Unicode checking.
type CheckOptions struct {
	Field            string
	UnicodePolicy    string // "fail" (default) | "warn" | "off"
	RequireSignature bool
	DigestOnly       bool
	AllowedSigners   string
	Path             string
}

// UnicodeFinding describes a suspicious invisible or BiDi-control character.
type UnicodeFinding struct {
	Line      int
	Col       int
	Codepoint rune
	Kind      string
}

func strPtr(s string) *string { return &s }

// ScanSuspiciousUnicode scans text for hidden prompt-injection / BiDi characters.
func ScanSuspiciousUnicode(text string) []UnicodeFinding {
	var findings []UnicodeFinding
	line, col := 1, 1
	idx := 0
	for _, r := range text {
		var kind string
		switch {
		case r >= 0xE0000 && r <= 0xE007F:
			kind = "unicode_tag"
		case (r >= 0x202A && r <= 0x202E) || (r >= 0x2066 && r <= 0x2069) || r == 0x061C || r == 0x200E || r == 0x200F:
			kind = "bidi_control"
		case r == 0x200B || r == 0x2060 || (r >= 0x2061 && r <= 0x2064):
			kind = "zero_width"
		case r == 0xFEFF && idx > 0:
			kind = "embedded_bom"
		}
		if kind != "" {
			findings = append(findings, UnicodeFinding{Line: line, Col: col, Codepoint: r, Kind: kind})
		}
		if r == '\n' {
			line++
			col = 1
		} else {
			col++
		}
		idx++
	}
	return findings
}

func formatUnicodeFindings(findings []UnicodeFinding) string {
	if len(findings) == 0 {
		return ""
	}
	f := findings[0]
	head := fmt.Sprintf("U+%04X (%s) at %d:%d", f.Codepoint, f.Kind, f.Line, f.Col)
	if len(findings) > 1 {
		return fmt.Sprintf("%s (+%d more)", head, len(findings)-1)
	}
	return head
}

// CheckDocumentText checks a sealed instruct markdown document (instruct-v2 FULL mode).
func CheckDocumentText(text string) Result {
	return CheckDocumentTextWith(text, CheckOptions{Field: SealField, UnicodePolicy: "fail"})
}

// CheckDocumentTextField is like CheckDocumentText with a custom seal field name.
func CheckDocumentTextField(text, field string) Result {
	return CheckDocumentTextWith(text, CheckOptions{Field: field, UnicodePolicy: "fail"})
}

// CheckDocumentTextWith checks an instruct document with full options.
func CheckDocumentTextWith(text string, opts CheckOptions) Result {
	field := opts.Field
	if field == "" {
		field = SealField
	}
	upol := opts.UnicodePolicy
	if upol == "" {
		upol = "fail"
	}
	if upol == "fail" {
		if u := ScanSuspiciousUnicode(text); len(u) > 0 {
			return Result{
				OK: false, Status: "suspicious_unicode",
				Message: strPtr(formatUnicodeFindings(u)),
			}
		}
	}

	doc := parseDocument(text)
	if !doc.hadFrontMatter {
		actual := computeDigest(doc, field)
		return Result{
			OK: false, Status: "missing_seal", Algorithm: strPtr("blake3"),
			Actual: strPtr(actual.Qualified), Message: strPtr("missing hashseal field"),
		}
	}
	sealRaw := extractReservedField(doc.fmLines, field)
	if sealRaw == nil {
		actual := computeDigest(doc, field)
		return Result{
			OK: false, Status: "missing_seal", Algorithm: strPtr("blake3"),
			Actual: strPtr(actual.Qualified), Message: strPtr("missing hashseal field"),
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
	actual := computeDigest(doc, field)
	if actual.Hex != expected.Hex || actual.Algorithm != expected.Algorithm {
		return Result{
			OK: false, Status: "mismatch", Algorithm: strPtr(expected.Algorithm),
			Expected: strPtr(expected.Qualified), Actual: strPtr(actual.Qualified),
		}
	}

	sig := extractReservedField(doc.fmLines, SigField)
	if opts.RequireSignature && sig == nil {
		return Result{
			OK: false, Status: "missing_signature", Algorithm: strPtr(actual.Algorithm),
			Expected: strPtr(expected.Qualified), Message: strPtr("missing hashseal_sig"),
		}
	}

	if sig != nil && !opts.DigestOnly && (opts.AllowedSigners != "" || opts.RequireSignature) {
		declaredScope := extractReservedField(doc.fmLines, ScopeField)
		if declaredScope != nil && opts.Path != "" && *declaredScope != "*" && *declaredScope != opts.Path {
			return Result{
				OK: false, Status: "bad_signature", Algorithm: strPtr(actual.Algorithm),
				Expected: strPtr(expected.Qualified),
				Message:  strPtr(fmt.Sprintf("signature scope mismatch: signed for '%s', checked at '%s'", *declaredScope, opts.Path)),
			}
		}
		verifyScope := "*"
		if declaredScope != nil {
			verifyScope = *declaredScope
		} else if opts.Path != "" {
			verifyScope = opts.Path
		}
		if opts.AllowedSigners == "" {
			return Result{
				OK: false, Status: "unverified_signature", Algorithm: strPtr(actual.Algorithm),
				Expected: strPtr(expected.Qualified), Message: strPtr("missing AllowedSigners for signature verification"),
			}
		}
		sigPayload := fmt.Sprintf("HASHSEAL-SIG-V2\ndigest=%s\nscope=%s\n", actual.Qualified, verifyScope)
		status, signer, msg := VerifySSHSig([]byte(sigPayload), *sig, opts.AllowedSigners, "hashseal")
		if status == "good" {
			return Result{
				OK: true, Status: "valid", Algorithm: strPtr(actual.Algorithm),
				Expected: strPtr(expected.Qualified), Actual: strPtr(actual.Qualified),
				Signer: strPtr(signer),
			}
		}
		return Result{
			OK: false, Status: status, Algorithm: strPtr(actual.Algorithm),
			Expected: strPtr(expected.Qualified), Message: strPtr(msg),
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
	return strings.TrimPrefix(s, "\ufeff")
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
				body := afterOpen[offset+idx+1+3:]
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

func isReservedKey(key, sealField string) bool {
	return key == sealField ||
		key == SealField ||
		key == SigField ||
		key == SigFormatField ||
		key == SignerField ||
		key == ScopeField ||
		key == KeyIDField
}

func trimLeadingSpaceTab(s string) string {
	i := 0
	for i < len(s) && (s[i] == ' ' || s[i] == '\t') {
		i++
	}
	return s[i:]
}

func extractReservedField(lines []string, field string) *string {
	i := 0
	n := len(lines)
	for i < n {
		if !strings.HasPrefix(lines[i], " ") && !strings.HasPrefix(lines[i], "\t") {
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
		}
		i++
	}
	return nil
}

func stripReservedFmLines(lines []string, sealField string) []string {
	out := make([]string, 0, len(lines))
	i := 0
	n := len(lines)
	for i < n {
		trimmed := strings.TrimSpace(lines[i])
		colon := strings.Index(trimmed, ":")
		if colon >= 0 && !strings.HasPrefix(lines[i], " ") && !strings.HasPrefix(lines[i], "\t") && isReservedKey(strings.TrimSpace(trimmed[:colon]), sealField) {
			i++
			for i < n {
				L := lines[i]
				if strings.HasPrefix(L, " ") || strings.HasPrefix(L, "\t") {
					i++
					continue
				}
				if strings.TrimSpace(L) == "" && i+1 < n && (strings.HasPrefix(lines[i+1], " ") || strings.HasPrefix(lines[i+1], "\t")) {
					i++
					continue
				}
				break
			}
			continue
		}
		out = append(out, lines[i])
		i++
	}
	return out
}

func hashPayload(doc parsedDoc, sealField string) []byte {
	bodyLf := normalizeLf(doc.bodyRaw)
	kept := stripReservedFmLines(doc.fmLines, sealField)
	hasNonEmpty := false
	for _, l := range kept {
		if strings.TrimSpace(l) != "" {
			hasNonEmpty = true
			break
		}
	}
	if hasNonEmpty {
		return []byte(instructV2Domain + "\nFM\n" + strings.Join(kept, "\n") + "\nBODY\n" + bodyLf)
	}
	return []byte(instructV2Domain + "\nBODY\n" + bodyLf)
}

func computeDigest(doc parsedDoc, sealField string) Digest {
	return Blake3Digest(hashPayload(doc, sealField))
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
