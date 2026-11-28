Output docs/assets/demos/ssh-signing.gif
Output docs/assets/demos/ssh-signing.webm

Set Shell "bash"
Set Theme "Catppuccin Mocha"
Set FontSize 15
Set Width 1000
Set Height 560
Set Padding 20
Set TypingSpeed 30ms

Hide
Type "export PATH=/vhs/target/x86_64-unknown-linux-musl/release:$PATH"
Enter
Type "export PS1='$ '"
Enter
Type 'DEMO_DIR=$(mktemp -d) && cd "$DEMO_DIR" && git init -q'
Enter
Type 'ssh-keygen -q -t ed25519 -N "" -f "$DEMO_DIR/demo_key" -C "demo@hashseal.ai"'
Enter
Type 'mkdir -p .hashseal && printf "demo@hashseal.ai namespaces=\"hashseal\" %s\n" "$(cat "$DEMO_DIR/demo_key.pub")" > .hashseal/allowed_signers'
Enter
Type 'git config gpg.format ssh && git config user.signingkey "$DEMO_DIR/demo_key"'
Enter
Type 'printf "# Agent Instructions\n\nRun cargo test --workspace --release.\n" > AGENTS.md'
Enter
Type 'printf "# Benign Skill\n\nSummarize git status.\n" > SKILL.md'
Enter
Type "clear"
Enter
Show

Type "hashseal seal --instruct --sign --lock --root ."
Enter
Sleep 1.5s

Type "hashseal check --root . --require-signature"
Enter
Sleep 1.2s

Type "cp SKILL.md AGENTS.md"
Enter
Sleep 600ms

Type "hashseal check --root . --require-signature"
Enter
Sleep 1.8s

Screenshot docs/assets/demos/ssh-signing.png
Sleep 1s
