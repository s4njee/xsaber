# xsaber

xsaber is a cross-platform, dual-pane file-transfer client. It speaks xsync
v3 over an existing russh SSH session when the remote has `xs`, and falls back
to SFTP when it does not. An embedded SSH terminal uses the same session.
FTP/FTPS, S3, WebDAV, SMB and NFS are not v1 protocols.

The application is a Rust workspace scaffold for the planned GPUI Kit shell:

```
crates/engine  rules, sessions, remote filesystems and transfers (no GPUI)
crates/term    terminal emulator model (no GPUI)
crates/cli     xsb headless driver
crates/app     GPUI shell
```

xsaber is the suite's third GUI. It is intentionally a port of the reusable
pieces of `excalibur/crates/engine`, not a Cargo dependency on that crate. The
xsync v3 seam and the port boundary are documented in
[`integration.md`](integration.md); the provider-side prerequisites are linked
there by story id.

## Build and test

The local workspace scaffold is present. Run the following from this directory
(the `xsync` sibling is required by the path dependency):

```bash
cargo +1.98.0 build --workspace
cargo +1.98.0 test --workspace
cargo +1.98.0 fmt --all -- --check
```

The 2026-09-08 local gate passed `build --workspace`, `test --workspace` (two
engine tests), `fmt --all -- --check`, `clippy --workspace --all-targets --
-D warnings`, `cargo deny check` (warnings only), `cargo audit`, and
`git diff --check`. The CI workflow passes local `actionlint` but has not run
on GitHub. Its package job intentionally compiles the app in release mode; it
does not produce or verify a native bundle. Bundle tooling and native launch
checks are E16 work.

The current headless entry point is:

```bash
cargo +1.98.0 run -p xsb -- --help
```

The `examples/kitchen-sink` visual fixture is planned by E1-S6; when it lands,
it will use mock data and make no network connection. Desktop packaging and
native-platform verification are E16 work;
Windows is kept in CI but is not a v1 release target.

The nested repository is on local branch `xcalibur` with origin
`https://github.com/s4njee/xsaber`, but it has no commit or push yet. E0-S1
owns publishing that branch; root `.gitmodules` registration and the
superproject pin remain `E17-S1` work.

## Planning documents

`README.md`, [`backlog.md`](backlog.md), and
[`integration.md`](integration.md) are the live documents. [`plan.md`](plan.md)
is the greenfield reference for story definitions and decisions until the
stories have landed; it is intentionally retained as the exception to the
usual three-document rule. Design deviations are recorded in
[`docs/design-gaps.md`](docs/design-gaps.md), and architectural decisions are
in [`docs/decisions/`](docs/decisions/).
