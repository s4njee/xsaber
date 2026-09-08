# ADR-0004: Port selected engine patterns instead of depending on excalibur

- Date: 2026-09-08
- Status: Accepted

## Context

excalibur already has useful `Surface` errors, host-key handling, secret
wrappers and a transfer queue. xsaber needs the same safety shape, but the
engine is unpublished, unversioned and coupled to excalibur's broader backend
set and path dependencies.

## Decision

Port `error.rs`, `hostkeys/`, `secrets/` and the transfer-queue shape into
`xsaber-engine` with attribution. Keep a Tauri-free engine, and do not add a
Cargo dependency on `../excalibur/crates/engine`. Record source commits and
drift tooling under `E17-S4`; revisit extraction only through the suite's
shared-crate decision `E17-S3`.

## Consequences

xsaber can evolve its SSH/xsync/SFTP contracts independently and keeps its
engine free of Tauri and GPUI. The port can temporarily drift, so each copied
file needs provenance and fixes must be cross-applied until a shared-crate
decision is made.

## Alternatives rejected and evidence

- A path dependency was rejected because excalibur-engine is unversioned,
  carries four xsync path dependencies and includes an optional GPL-tainting
  SMB backend.
- A shared crate now was rejected because the suite's documented trigger is a
  decision after the third GUI exists (`E17-S3`), not an implicit rewrite.
- Copying all of excalibur was rejected because xsaber's provider set and
  session model are narrower; only the named safety/queue patterns are in
  scope.
