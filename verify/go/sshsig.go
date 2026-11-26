// Pure-Go OpenSSH SSHSIG parser + Ed25519 verifier (stdlib only).
// Copyright (c) 2026 MonkeyKing.dev
package hashseal

import (
	"bytes"
	"crypto/ed25519"
	"crypto/sha256"
	"crypto/sha512"
	"encoding/base64"
	"encoding/binary"
	"errors"
	"fmt"
	"strings"
)

type sshReader struct {
	b   []byte
	off int
}

func (r *sshReader) rem() int { return len(r.b) - r.off }

func (r *sshReader) readBytes(n int) ([]byte, error) {
	if r.off+n > len(r.b) {
		return nil, errors.New("unexpected EOF in SSHSIG")
	}
	s := r.b[r.off : r.off+n]
	r.off += n
	return s, nil
}

func (r *sshReader) readU32() (uint32, error) {
	b, err := r.readBytes(4)
	if err != nil {
		return 0, err
	}
	return binary.BigEndian.Uint32(b), nil
}

func (r *sshReader) readString() ([]byte, error) {
	n, err := r.readU32()
	if err != nil {
		return nil, err
	}
	return r.readBytes(int(n))
}

func writeSSHString(buf *bytes.Buffer, b []byte) {
	var lenBuf [4]byte
	binary.BigEndian.PutUint32(lenBuf[:], uint32(len(b)))
	buf.Write(lenBuf[:])
	buf.Write(b)
}

type parsedSSHSig struct {
	pubBlob   []byte
	pubKey    ed25519.PublicKey
	namespace string
	reserved  []byte
	hashAlg   string
	sigBytes  []byte
}

func parseSSHSig(armor string) (*parsedSSHSig, error) {
	norm := strings.ReplaceAll(armor, "\r\n", "\n")
	lines := strings.Split(norm, "\n")
	start, end := -1, -1
	for i, l := range lines {
		t := strings.TrimSpace(l)
		if t == "-----BEGIN SSH SIGNATURE-----" && start < 0 {
			start = i
		} else if t == "-----END SSH SIGNATURE-----" && start >= 0 {
			end = i
			break
		}
	}
	if start < 0 || end <= start {
		return nil, errors.New("invalid SSH SIGNATURE armor")
	}
	var b64 strings.Builder
	for _, l := range lines[start+1 : end] {
		b64.WriteString(strings.TrimSpace(l))
	}
	if b64.Len() == 0 {
		return nil, errors.New("empty SSH SIGNATURE armor")
	}
	raw, err := base64.StdEncoding.DecodeString(b64.String())
	if err != nil {
		return nil, err
	}

	r := &sshReader{b: raw}
	magic, err := r.readBytes(6)
	if err != nil || string(magic) != "SSHSIG" {
		return nil, errors.New("bad SSHSIG magic")
	}
	ver, err := r.readU32()
	if err != nil || ver != 1 {
		return nil, fmt.Errorf("unsupported SSHSIG version %d", ver)
	}
	pubBlob, err := r.readString()
	if err != nil {
		return nil, err
	}
	nsBytes, err := r.readString()
	if err != nil {
		return nil, err
	}
	reserved, err := r.readString()
	if err != nil {
		return nil, err
	}
	hashAlgBytes, err := r.readString()
	if err != nil {
		return nil, err
	}
	sigBlob, err := r.readString()
	if err != nil {
		return nil, err
	}
	if r.rem() != 0 {
		return nil, errors.New("trailing bytes in SSHSIG")
	}

	pr := &sshReader{b: pubBlob}
	pubType, err := pr.readString()
	if err != nil || string(pubType) != "ssh-ed25519" {
		return nil, fmt.Errorf("unsupported SSH key type: %s", string(pubType))
	}
	pubKey, err := pr.readString()
	if err != nil || len(pubKey) != ed25519.PublicKeySize {
		return nil, errors.New("invalid Ed25519 public key")
	}

	sr := &sshReader{b: sigBlob}
	sigType, err := sr.readString()
	if err != nil || string(sigType) != "ssh-ed25519" {
		return nil, fmt.Errorf("unsupported SSH sig type: %s", string(sigType))
	}
	sigBytes, err := sr.readString()
	if err != nil || len(sigBytes) != ed25519.SignatureSize {
		return nil, errors.New("invalid Ed25519 signature")
	}

	return &parsedSSHSig{
		pubBlob:   pubBlob,
		pubKey:    ed25519.PublicKey(pubKey),
		namespace: string(nsBytes),
		reserved:  reserved,
		hashAlg:   string(hashAlgBytes),
		sigBytes:  sigBytes,
	}, nil
}

type allowedSigner struct {
	principal string
	keyBlob   []byte
}

func parseAllowedSigners(text string) []allowedSigner {
	var out []allowedSigner
	for _, rawLine := range strings.Split(strings.ReplaceAll(text, "\r\n", "\n"), "\n") {
		line := strings.TrimSpace(rawLine)
		if line == "" || strings.HasPrefix(line, "#") {
			continue
		}
		parts := strings.Fields(line)
		if len(parts) < 3 {
			continue
		}
		principals := strings.Split(parts[0], ",")
		idx := 1
		if !strings.HasPrefix(parts[idx], "ssh-") && !strings.HasPrefix(parts[idx], "ecdsa-") && !strings.HasPrefix(parts[idx], "sk-") {
			idx++
		}
		if idx+1 >= len(parts) || parts[idx] != "ssh-ed25519" {
			continue
		}
		blob, err := base64.StdEncoding.DecodeString(parts[idx+1])
		if err != nil {
			continue
		}
		for _, p := range principals {
			p = strings.TrimSpace(p)
			if p != "" {
				out = append(out, allowedSigner{principal: p, keyBlob: blob})
			}
		}
	}
	return out
}

// VerifySSHSig verifies an OpenSSH SSHSIG over payload against allowedSignersText.
func VerifySSHSig(payload []byte, armor, allowedSignersText, namespace string) (status string, signer string, msg string) {
	parsed, err := parseSSHSig(armor)
	if err != nil {
		return "bad_signature", "", err.Error()
	}
	if parsed.namespace != namespace {
		return "bad_signature", "", fmt.Sprintf("namespace mismatch: got %s, want %s", parsed.namespace, namespace)
	}
	signers := parseAllowedSigners(allowedSignersText)
	var matched string
	for _, s := range signers {
		if bytes.Equal(s.keyBlob, parsed.pubBlob) {
			matched = s.principal
			break
		}
	}
	if matched == "" {
		return "untrusted_key", "", "signer key not in allowed_signers"
	}

	var hashDigest []byte
	switch parsed.hashAlg {
	case "sha512":
		sum := sha512.Sum512(payload)
		hashDigest = sum[:]
	case "sha256":
		sum := sha256.Sum256(payload)
		hashDigest = sum[:]
	default:
		return "bad_signature", "", "unsupported SSHSIG hash: " + parsed.hashAlg
	}

	var signed bytes.Buffer
	signed.WriteString("SSHSIG")
	writeSSHString(&signed, []byte(parsed.namespace))
	writeSSHString(&signed, parsed.reserved)
	writeSSHString(&signed, []byte(parsed.hashAlg))
	writeSSHString(&signed, hashDigest)

	if !ed25519.Verify(parsed.pubKey, signed.Bytes(), parsed.sigBytes) {
		return "bad_signature", "", "Ed25519 SSHSIG verification failed"
	}
	return "good", matched, ""
}
