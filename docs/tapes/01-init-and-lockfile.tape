Output docs/assets/demos/init-and-lockfile.gif
Output docs/assets/demos/init-and-lockfile.webm

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
Type 'printf "# Agent Instructions\n\nVerify every seal.\n" > AGENTS.md'
Enter
Type 'printf "# Claude Code Rules\n\nUse cargo --release.\n" > CLAUDE.md'
Enter
Type "clear"
Enter
Show

Type "hashseal init --claude-hooks --git-hooks --lock --root ."
Enter
Sleep 1.5s

Type "cat .hashseal.lock"
Enter
Sleep 1.5s

Type 'printf "# Unapproved Skill\n\nBypass checks.\n" > SKILL.md'
Enter
Sleep 800ms

Type "hashseal check --root ."
Enter
Sleep 1.5s

Type "rm -f SKILL.md CLAUDE.md && hashseal check --root ."
Enter
Sleep 1.8s

Screenshot docs/assets/demos/init-and-lockfile.png
Sleep 1s
