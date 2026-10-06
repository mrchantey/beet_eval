# beet_eval workflows.

# `rustfmt.toml` uses nightly-only options, so formatting needs a nightly
# toolchain. Pinned to match the `eval` beet worktree so both trees format
# identically; bump the two together.
fmt-toolchain := 'nightly-2026-07-02'

# recipe args reach the command verbatim (`"$@"`), so a quoted value holding a
# `>` or a space is one argument and never a shell redirect
set positional-arguments

# List recipes.
default:
	@just --list

#     just cli --help
#
# This repo's beet cli: the stock runner plus `EvalPlugin`, serving `main.bsx`.
cli *args:
	cargo run --features=cli -- "$@"

# Format with the pinned nightly. Never `cargo fmt`.
fmt *args:
	#!/usr/bin/env bash
	set -euo pipefail
	# bare `cargo fmt` on stable silently drops every nightly-only option in
	# `rustfmt.toml`, reformatting the whole tree into a huge bogus diff.
	rustup toolchain list | grep -q '^{{ fmt-toolchain }}' \
		|| rustup toolchain install {{ fmt-toolchain }} --profile minimal --component rustfmt
	# `-p`, not `--all`: `--all` also formats path dependencies, ie the beet worktree
	cargo +{{ fmt-toolchain }} fmt -p beet_eval "$@"

# The native suite (the beet harness; pass `--snap` to update snapshots).
test *args:
	cargo test "$@"
