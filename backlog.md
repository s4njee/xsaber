# xsaber backlog — open work and completed evidence

Status snapshot: 2026-09-08. This file records unfinished work and retains
completed closure evidence; the original story definitions and acceptance
criteria remain in [`plan.md`](plan.md).

## Status snapshot

- **Milestone:** M0 — E0 complete; E1 design-system work is in progress.
- **Branch:** published `xcalibur` branch at `dd069e8` (E2 SSH policy); origin is `https://github.com/s4njee/xsaber`. Root registration and pinning remain `E17-S1` work.
- **Last 5 commits:** `dd069e8` Add SSH authentication and config policies; `cc3a57e` Add SSH session and host-key foundations; `d8234c3` Clarify completed work and blockers; `fa13824` Record E1 progress; `9ccd066` Add E1 theme foundation.
- **Test gate:** the clean recursive suite checkout at `/tmp/xsaber-e0-clean.83Ddis` built and tested xsaber against the published suite xsync pin, with 4 engine tests passing. Local build/test, workspace-scoped fmt, clippy, `cargo deny check`, `cargo audit`, release-mode app build, `actionlint`, and `git diff --check` pass on 2026-09-08. GitHub CI run [34194549604](https://github.com/s4njee/xsaber/actions/runs/34194549604) passed its macOS and Ubuntu Rust jobs, supply-chain job, and macOS/Ubuntu release app builds; the Windows job reported the known `xsync-core` `rustix::fs` failure and is allowed by E0 until `E16-S4`. The package job is a release-mode app compilation, not native bundle verification.
- **Not gated:** `E17-S1` submodule registration, `.gitmodules`, and the superproject SHA pin; native bundles remain E16 work and native Windows packaging remains `E16-S4`.

## Completed work: E0 — 2026-09-08

E0 is complete. E17-S1 is a separate closure for root registration and
pinning; E0-S1 explicitly excludes changing `.gitmodules` or the superproject
pin.

- **Published revision:** `85876b3` on `xcalibur` at `https://github.com/s4njee/xsaber` (`94d66ed` is the implementation commit).
- **Clean-checkout evidence:** `/tmp/xsaber-e0-clean.83Ddis`, recursive suite checkout, Rust 1.98.0; workspace build and test passed with 4 engine tests.
- **Local evidence:** workspace-scoped formatting, clippy with warnings denied, cargo-deny, cargo-audit, release-mode app build, actionlint, and diff check passed.
- **GitHub evidence:** [run 34194549604](https://github.com/s4njee/xsaber/actions/runs/34194549604) passed macOS/Ubuntu Rust, supply-chain, and macOS/Ubuntu release app jobs. Windows reported the `xsync-core` `rustix::fs` failure; E0 allows that result until `E16-S4`.
- **Package gate:** macOS and Ubuntu release-mode app compilations passed; this is not native bundle verification.
- **Root registration/pinned SHA:** pending `E17-S1` and intentionally outside E0.

### E0 story status

| Story | Status | Evidence |
|---|---|---|
| `E0-S1` Repository and workspace | Done — 2026-09-08 | Published `xcalibur` branch at `85876b3`; clean recursive checkout built/tested with Rust 1.98.0. Root registration/pinning remains `E17-S1`. |
| `E0-S2` CI | Done — 2026-09-08 | Workflow actionlint and local gates pass; GitHub run 34194549604 passed macOS/Ubuntu Rust, supply-chain, and release app jobs. Windows reported the known allowed `rustix::fs` failure. |
| `E0-S3` Three live docs | Done — 2026-09-08 | `README.md`, `backlog.md`, `integration.md`, and suite orientation are present and updated with the published evidence. |
| `E0-S4` ADRs | Done — 2026-09-08 | Five ADRs are present under `docs/decisions/`, covering GPUI Kit, xsync/SFTP, terminal, engine port, and ring. |
| `E0-S5` Design gaps ledger | Done — 2026-09-08 | Seeded deviations are recorded in `docs/design-gaps.md`; later departures must append dated rows. |
| `E0-S6` Logging and diagnostics | Done — 2026-09-08 | Four engine tests pass: all three secret kinds are redacted in spans, diagnostics snapshot/subscription is bounded, `XSABER_LOG=debug` raises verbosity, and rolling files are created under app data. |

## E1 — Design system in GPUI Kit

| Story | Status |
|---|---|
| `E1-S1` Theme | In progress — theme asset, typed app tokens, and `ThemeSet` JSON round-trip test landed in `9ccd066`; startup registry loading remains. |
| `E1-S2` Fonts | In progress — OFL font assets and provenance landed in `80d4cce`; `AssetSource`, font registration, and fallback-independent render proof remain. |
| `E1-S3` Icons and wordmark | Planned |
| `E1-S4` Density primitives | Planned |
| `E1-S5` Window chrome and menus | Planned |
| `E1-S6` Kitchen-sink example app | Planned |

### Completed E1 slices

- [x] `E1-S1` theme asset and app-owned design tokens — `9ccd066`; the
  focused `ThemeSet` round-trip and token tests pass.
- [x] `E1-S2` licensed local font assets and provenance — `80d4cce`; Barlow
  400/500/600/700 and IBM Plex Mono 400 plus their OFL texts are bundled.

These checked slices do not close their parent stories: the unfulfilled ACs
remain in the status table above and in `plan.md`.

### Current blockers

- `E1-S2` cannot close its bundle-shipping AC until `E16` supplies native
  bundles. Runtime `AssetSource` and font-registration work is still local
  implementation work, not an external blocker.
- `E17-S2` is blocked on xsync assigning and scoping two missing provider
  follow-up IDs: server-side recursive delete/parent-creating mkdir, and a
  paged overflow alternative for `read_dir_path`. The exact consumer
  workarounds are recorded in [`integration.md`](integration.md).
- `E16-S4` is blocked by the known Windows provider failure: `xsync-core`
  imports Unix-only `rustix::fs`. The E0 CI job reports this failure as
  allowed; it must be resolved before the Windows release AC can close.

## Later story identity

The following unfinished story ranges remain owned by `plan.md`; this compact
index keeps their IDs stable while work is sequenced behind M0:

| Epic | Stories |
|---|---|
| `E2` SSH sessions | `E2-S1`–`E2-S5` |
| `E3` xsync v3 remote filesystem | `E3-S0`–`E3-S4` |
| `E4` local filesystem | `E4-S1`–`E4-S3` |
| `E5` remote/local navigation | `E5-S1`–`E5-S2` |
| `E6` transfer engine and queue | `E6-S1`–`E6-S9` |
| `E7` terminal drawer | `E7-S1`–`E7-S5` |
| `E8` conflicts and recovery | `E8-S1`–`E8-S5` |
| `E9` sites and bookmarks | `E9-S1`–`E9-S6` |
| `E10` diagnostics and history | `E10-S1`–`E10-S4` |
| `E11` settings and preferences | `E11-S1`–`E11-S4` |
| `E12` accessibility and input | `E12-S1`–`E12-S3` |
| `E13` terminal and transfer integration | `E13-S1`–`E13-S2` |
| `E14` performance | `E14-S1` |
| `E15` test fixtures and verification | `E15-S1`–`E15-S4` |
| `E16` packaging and release | `E16-S1`–`E16-S4` |
| `E17` suite integration | `E17-S1`–`E17-S4` |

`E3-S0` and `E17-S2` are prerequisite records, not permission to silently
change xsync. Their provider/consumer contract is in
[`integration.md`](integration.md).

## E2 — SSH sessions

| Story | Status |
|---|---|
| `E2-S1` Session actor | In progress — `SessionConfig`, bounded event bus, elapsed handshake events, and terminal lifecycle state landed in `cc3a57e`; a live russh actor and E15 concurrent-channel test remain. |
| `E2-S2` Host-key trust | In progress — app-private TOFU store, atomic writes, revoked/changed-key handling, and SHA-256 fingerprints landed in `cc3a57e`; russh callback wiring and host-certificate CA verification remain. |
| `E2-S3` Authentication ladder | In progress — typed agent/key/password/KI policy, partial-success continuation, and secret-safe prompt types landed in `dd069e8`; live russh authentication and E15 server proof remain. |
| `E2-S4` SSH config | In progress — safe, typed Host/HostName/User/Port/IdentityFile/ProxyCommand resolution and explicit ProxyJump/Match errors landed in `dd069e8`; mandated `russh-config::parse_home` and ProxyCommand stream wiring remain. |
| `E2-S5` Reconnect supervisor | Planned |

## E3/E4 — Remote filesystem and SFTP fallback

| Story | Status |
|---|---|
| `E3-S1` `RemoteFs` trait | In progress — object-safe contract, safe remote paths, metadata, capability bits, and I/O-handle interfaces landed in `f9b9a8c`; adapter implementations remain. |
| `E3-S2` xsync backend | Blocked — requires E2’s live russh actor and E15’s in-process SSH fixture before an exec-channel backend can be tested safely. |
| `E3-S3` Capability gating | In progress — `Caps` preserves unknown bits and exposes only explicit capabilities in `f9b9a8c`; mapping real xsync grants remains. |
| `E3-S4` Remote binary override | Planned |
| `E4-S1` SFTP backend | Blocked — requires the live E2 session/channel actor and `russh-sftp` integration; performance proof also requires E15 netem/Docker fixtures. |
| `E4-S2` Shared trust and auth | Blocked — depends on E2’s live russh callback/auth adapter. |
| `E4-S3` Protocol selection | In progress — typed Auto/Xsync/SFTP selection and fallback-only-on-unavailability policy landed in `f9b9a8c`; backend execution remains. |
