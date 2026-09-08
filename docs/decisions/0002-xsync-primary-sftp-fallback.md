# ADR-0002: Use xsync v3 first and SFTP as the fallback

- Date: 2026-09-08
- Status: Accepted

## Context

The suite already owns xsync v3 and its async `xsync-client` API. xsaber needs
random reads, verified writes, resumable staging and a shared SSH session;
some hosts will not have `xs` installed.

## Decision

Open `xs --server <root>` over the authenticated russh channel and construct
the client with `Client::from_stream`. If the remote binary is unavailable or
the v3 handshake cannot be established, degrade to `russh-sftp` on the same
SSH connection. Expose the actual backend in capabilities and the protocol
badge.

## Consequences

The main path gets xsync's BLAKE3 verification, stage resume tokens and async
file API without a second SSH process. SFTP remains useful and interoperable,
but its rename-over-existing operation is not atomic in `russh-sftp` and its
read pipeline needs separate care. Backend-specific capabilities must never be
silently presented as universal.

## Alternatives rejected and evidence

- FTP/FTPS were rejected as v1 protocols; the design handoff's FTP label is a
  documented design gap.
- A second `ssh` process using `run_client_push` was rejected: research found
  it is blocking, creates another session, emits single-lane progress and does
  not populate remote `ProgressWatch`.
- SFTP-only was rejected because it would discard xsync's v3 random-access,
  staging and verification surface.
- A new daemon transport was rejected for v1; xsync's documented
  `AsyncRead + AsyncWrite`/`from_stream` seam already fits russh.
