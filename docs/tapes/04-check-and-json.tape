Output docs/assets/demos/check-and-json.gif
Output docs/assets/demos/check-and-json.webm

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
Type 'DEMO_DIR=$(mktemp -d) && cd "$DEMO_DIR"'
Enter
Type 'printf "# Agent Instructions\n\nVerify before running.\n" > AGENTS.md'
Enter
Type "hashseal seal --instruct --root . >/dev/null"
Enter
Type "clear"
Enter
Show

Type "hashseal-check --root . --digest-only"
Enter
Sleep 1.2s

Type "hashseal check --root . --format json"
Enter
Sleep 1.8s

Screenshot docs/assets/demos/check-and-json.png
Sleep 1s
