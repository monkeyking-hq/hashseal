Output docs/assets/demos/tamper-and-unicode.gif
Output docs/assets/demos/tamper-and-unicode.webm

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
Type 'printf -- "---\nglobs:\n  - \"src/**\"\ntools:\n  - \"read\"\n---\n# Agent Policy\n\nOnly read files under src/.\n" > AGENTS.md'
Enter
Type "clear"
Enter
Show

Type "hashseal seal --instruct --root ."
Enter
Sleep 1.2s

Type "cp AGENTS.md AGENTS.md.bak"
Enter
Sleep 500ms

Type 'sed -i "s/\"read\"/\"bash\"/" AGENTS.md && hashseal check --root .'
Enter
Sleep 1.5s

Type 'mv AGENTS.md.bak AGENTS.md && printf "Hidden\xe2\x80\xaeoverride\n" >> AGENTS.md'
Enter
Sleep 1s

Type "hashseal check --root ."
Enter
Sleep 1.8s

Screenshot docs/assets/demos/tamper-and-unicode.png
Sleep 1s
