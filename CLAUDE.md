# CLAUDE.md

## Comments

Near-zero. Write a comment only where the code would otherwise read as a bug.
Names and types carry everything else.

Never write a comment that states:

- what the code does
- why the code changed, or what the previous implementation was
- why an alternative was rejected
- justification for a design decision

That material goes in the commit message, not the source. A doc comment on a
public item states what a caller must know to use it correctly — nothing about
how it got that way.

## Edits

Plan mode is the default for this project (`.claude/settings.local.json`).
Propose, get approval, then edit.

A question is a question. "Is this wasteful?", "can you find it?", "how would
you fix it?", "what's wrong with this?" get an answer and nothing else. Wait for
an explicit instruction to change the code.

## Build

Use `CARGO_TARGET_DIR=/home/claude/.cargo-target-cache/game` and do not vary
`WGPU_BACKEND`/`DISPLAY` between invocations — both cause full rebuilds.
