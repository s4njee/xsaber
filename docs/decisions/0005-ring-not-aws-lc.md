# ADR-0005: Select ring as russh's crypto backend

- Date: 2026-09-08
- Status: Accepted

## Context

russh supports crypto backends with different native build requirements. xsaber
must build on macOS, Linux and Windows CI under the suite's pinned Rust 1.98.0
toolchain, and it should align with excalibur's SSH dependency choices.

## Decision

Configure russh with `default-features = false` and the `ring`, `rsa` and
`flate2` features. Keep the exact russh/russh-sftp pins from the plan.

## Consequences

The build avoids the aws-lc-rs toolchain friction identified in the research
and keeps the two GUI SSH stacks aligned. Native crypto build and platform
verification remain CI obligations; this decision does not make Windows a v1
release target.

## Alternatives rejected and evidence

- `aws-lc-rs` was rejected for this slice because the plan's research found
  toolchain friction on Windows.
- The russh defaults were rejected because they select the unwanted backend
  transitively and make the platform matrix less predictable.
- A separate SSH library was rejected because russh already provides the
  required client handler, agent authentication, PTY and channel model and is
  the stack already carried by excalibur.
