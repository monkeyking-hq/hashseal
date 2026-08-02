---
hashseal: "blake3:492d0958259e58fa3104adda26e54a52683eb9570b3e60ae1cf23b21214f8ce9"
---
# hashseal-verify (Ruby)

Zero-dependency HashSeal instruct document verifier (stdlib + vendored pure-Ruby BLAKE3).

**Signed, Sealed, Delivered - I'm Yours.**

## Requirements

- Ruby 2.7+ (3.x / 4.x recommended)
- **No gems** — pure Ruby only

## Test (preferred — pure)

```bash
cd verify/ruby
ruby test/run_vectors.rb
```

Uses frozen vectors at `../vectors/instruct-v1.json` and `../vectors/tree-v1.json`.

```bash
ruby test/run_vectors.rb
ruby test/run_tree_vectors.rb
```

## API

```ruby
require_relative "lib/hashseal_check"
require_relative "lib/hashseal_tree"

r = Hashseal.check_document_text(markdown_text)
# r[:ok], r[:status]  # "valid"|"mismatch"|"missing_seal"|"invalid_format"

h = Hashseal.hash_tree_file_content("a.txt", "hello\n")
v = Hashseal.verify_tree_in_memory({ "a.txt" => "hello\n" }, [{ "path" => "a.txt", "digest" => h[:digest] }])
# v["ok"], v["findings"]  # every non-OK path named
```

Instruct: FULL canonical digest. Tree: LF text policy + in-memory verify (same as JS/core).

## Vendor

`vendor/blake3.rb` is a pure-Ruby port of the official BLAKE3 reference implementation
(CC0 / Apache-2.0 dual).

## Legacy bridge (optional)

`lib/hashseal_verify.rb` + `test_vectors.rb` shell to the Python check for cross-check only.
Prefer `lib/hashseal_check.rb` for zero external runtimes.

```text
Copyright (c) 2026 MonkeyKing.dev
```
