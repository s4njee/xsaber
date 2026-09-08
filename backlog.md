# xsaber backlog — open work and completed evidence

Status snapshot: 2026-09-08. This file records unfinished work and retains
completed closure evidence; the original story definitions and acceptance
criteria remain in [`plan.md`](plan.md).

## Status snapshot

- **Milestone:** M0 — E0 complete; E1 design-system stories are planned and not started.
- **Branch:** published `xcalibur` branch at `85876b3` (`94d66ed` scaffold plus the CI formatting fix); origin is `https://github.com/s4njee/xsaber`. Root registration and pinning remain `E17-S1` work.
- **Last 5 commits:** `85876b3` Scope formatting check to xsaber; `94d66ed` Scaffold xsaber workspace.
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
| `E1-S1` Theme | Planned |
| `E1-S2` Fonts | Planned |
| `E1-S3` Icons and wordmark | Planned |
| `E1-S4` Density primitives | Planned |
| `E1-S5` Window chrome and menus | Planned |
| `E1-S6` Kitchen-sink example app | Planned |

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
