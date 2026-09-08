# ADR-0003: Use alacritty_terminal for the embedded terminal model

- Date: 2026-09-08
- Status: Accepted

## Context

The terminal drawer must consume bytes from a russh PTY channel, parse ANSI
sequences, maintain a screen model and render through a GPUI element. It must
not own a second transport or runtime.

## Decision

Use `alacritty_terminal = 0.26.0` and feed its `vte::ansi::Processor` directly
from channel bytes. Use the terminal model only; do not use its `tty` module.

## Consequences

xsaber gets a maintained ANSI state machine and a model suitable for a custom
GPUI renderer. The project owns PTY sizing, input encoding and channel
lifecycle. Zed maintains a fork, so dependency updates may occasionally need a
small compatibility patch.

## Alternatives rejected and evidence

- `vt100` was rejected as unmaintained for this use.
- `termwiz`/`wezterm-term` were rejected because the researched API and
  publication state did not match the required direct `&[u8]` feed.
- `avt` was rejected because it did not provide the required input shape.
- alacritty's `tty` module was rejected because russh already owns the PTY and
  transport; using both would duplicate session responsibility.
