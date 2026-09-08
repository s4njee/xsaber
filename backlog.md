# xsaber backlog — open work only

Status snapshot: 2026-09-08. This file records unfinished work; the original
story definitions and acceptance criteria remain in [`plan.md`](plan.md).
No story below is marked Done while the implementation and acceptance evidence
are being assembled concurrently.

## Status snapshot

- **Milestone:** M0 — project scaffold and design system (`E0`, `E1`); not yet gated.
- **Branch:** local nested repository on `xcalibur`; origin is `https://github.com/s4njee/xsaber`, with no commit or push yet. E0-S1 owns the first publication; root registration and pinning remain `E17-S1` work.
- **Last 5 commits:** none; the nested repository has no commit yet.
- **Test gate:** local `build --workspace`, `test --workspace` (2 engine tests), `fmt --all -- --check`, clippy, `cargo deny check` (warnings only), `cargo audit`, and `git diff --check` pass on 2026-09-08. `actionlint` passes locally; GitHub CI has not run. The package job is a release-mode app compilation, not native bundle verification.
- **Not gated:** E0-S1 remote publication and GitHub CI; `E17-S1` submodule registration, `.gitmodules`, and the superproject SHA pin; native bundles remain E16 work and native Windows packaging remains `E16-S4`.

## E0 closure evidence (fill after publication)

E0 stays open until its local implementation is published and the checks below
have authoritative evidence. E17-S1 is a separate closure for root
registration and pinning.

- **Published xsaber commit:** _pending E0-S1 push_
- **Published branch:** `xcalibur`
- **Clean-checkout build/test evidence:** _pending; record checkout location, toolchain and commands_
- **GitHub CI run:** _pending; record the run URL and macOS/Linux results plus the reported Windows result_
- **Package gate evidence:** _pending; record the release-mode app compilation, not a native bundle claim_
- **Root registration/pinned SHA:** _pending E17-S1; do not use this field to extend E0-S1_

## E0 — Project scaffold and suite membership

| Story | Status | Remaining evidence |
|---|---|---|
| `E0-S1` Repository and workspace | In progress — locally built/tested | Workspace, toolchain, license and cargo-deny checks pass locally; publish the `xcalibur` branch and record a clean-checkout result. Root registration/pinning is excluded and remains `E17-S1`. |
| `E0-S2` CI | Implemented locally; unrun remotely | Workflow and actionlint pass locally; GitHub matrix, reported Windows result, and release-mode app compilation remain. Native bundle verification is explicitly E16 work. |
| `E0-S3` Three live docs | Implemented locally; acceptance open | Live docs and suite orientation are present; final review against the clean published scaffold remains. |
| `E0-S4` ADRs | Implemented locally; acceptance open | Five ADRs are seeded from plan research; final implementation review remains. |
| `E0-S5` Design gaps ledger | Implemented locally; ongoing | Seeded deviations are recorded; later departures must append dated rows. |
| `E0-S6` Logging and diagnostics | Implemented/tested locally; integration evidence open | One local test covers redaction of all three `Secret` kinds in spans; a second covers bounded diagnostics snapshots/subscription. `XSABER_LOG=debug` is implemented in `configured_filter` but is not directly tested yet; global logging integration, in-app sink consumption, and published CI evidence remain. |

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
