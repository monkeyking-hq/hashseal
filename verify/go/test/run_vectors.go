// Run official instruct-v1 vectors.
// Usage (from verify/go): go run ./test/
//
// Copyright (c) 2026 MonkeyKing.dev
package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	hashseal "github.com/hashseal/verify-go"
)

type expect struct {
	OK       *bool   `json:"ok"`
	Status   string  `json:"status"`
	Digest   string  `json:"digest"`
	Expected string  `json:"expected"`
	Actual   string  `json:"actual"`
}

type casus struct {
	ID         string `json:"id"`
	Kind       string `json:"kind"`
	Text       string `json:"text"`
	BytesUTF8  string `json:"bytes_utf8"`
	Expect     expect `json:"expect"`
}

type vectorDoc struct {
	Spec  string  `json:"spec"`
	Cases []casus `json:"cases"`
}

func main() {
	_, thisFile, _, _ := runtime.Caller(0)
	vectorsPath := filepath.Join(filepath.Dir(thisFile), "..", "..", "vectors", "instruct-v1.json")
	raw, err := os.ReadFile(vectorsPath)
	if err != nil {
		fmt.Fprintf(os.Stderr, "read vectors: %v\n", err)
		os.Exit(1)
	}
	var doc vectorDoc
	if err := json.Unmarshal(raw, &doc); err != nil {
		fmt.Fprintf(os.Stderr, "parse vectors: %v\n", err)
		os.Exit(1)
	}
	if doc.Spec != "instruct-v1" {
		fmt.Fprintf(os.Stderr, "unexpected spec %s\n", doc.Spec)
		os.Exit(1)
	}

	passed, failed := 0, 0
	for _, c := range doc.Cases {
		err := runCase(c)
		if err != nil {
			failed++
			fmt.Fprintf(os.Stderr, "FAIL %s: %v\n", c.ID, err)
		} else {
			passed++
			fmt.Printf("ok  %s\n", c.ID)
		}
	}
	fmt.Printf("\n%d passed, %d failed\n", passed, failed)
	if failed != 0 {
		os.Exit(1)
	}
}

func runCase(c casus) error {
	switch c.Kind {
	case "raw_digest":
		actual := hashseal.Blake3DigestString(c.BytesUTF8).Qualified
		return assertEq(actual, c.Expect.Digest, c.ID+" digest")
	case "check":
		r := hashseal.CheckDocumentText(c.Text)
		if c.Expect.OK != nil {
			if r.OK != *c.Expect.OK {
				return fmt.Errorf("%s ok: got %v want %v", c.ID, r.OK, *c.Expect.OK)
			}
		}
		if err := assertEq(r.Status, c.Expect.Status, c.ID+" status"); err != nil {
			return err
		}
		if c.Expect.Digest != "" {
			act := ""
			if r.Actual != nil {
				act = *r.Actual
			}
			if err := assertEq(act, c.Expect.Digest, c.ID+" actual digest"); err != nil {
				return err
			}
			if r.OK {
				exp := ""
				if r.Expected != nil {
					exp = *r.Expected
				}
				if err := assertEq(exp, c.Expect.Digest, c.ID+" expected digest"); err != nil {
					return err
				}
			}
		}
		if c.Expect.Expected != "" {
			exp := ""
			if r.Expected != nil {
				exp = *r.Expected
			}
			if err := assertEq(exp, c.Expect.Expected, c.ID+" expected"); err != nil {
				return err
			}
		}
		if c.Expect.Actual != "" {
			act := ""
			if r.Actual != nil {
				act = *r.Actual
			}
			if err := assertEq(act, c.Expect.Actual, c.ID+" actual"); err != nil {
				return err
			}
		}
		return nil
	default:
		return fmt.Errorf("unknown kind %s", c.Kind)
	}
}

func assertEq(a, b, label string) error {
	if a != b {
		return fmt.Errorf("%s: got %q want %q", label, a, b)
	}
	return nil
}
