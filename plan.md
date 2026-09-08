# xsaber — plan

**Date:** 2026-09-08
**Scope:** a cross-platform, dual-pane file-transfer desktop client with an
embedded SSH terminal drawer, built in Rust on GPUI via GPUI Kit, speaking
xsync v3 as its primary remote protocol over a russh SSH session, with SFTP as
the fallback. Sixth project in the xcalibur suite.

This is the greenfield planning document. Per suite convention the three live
documents are `README.md`, `backlog.md` and `integration.md`; `plan.md` remains
the reference for story ids, acceptance criteria and decisions while completed
and unfinished work is tracked in `backlog.md`.

The design reference is `design_handoff_xsaber_ftp_client/` (five screens,
`1a`–`1e`). It is reference, not spec (`../AGENTS.md`); deviations are recorded
in `docs/design-gaps.md` (E0-S5). The mock calls the product an "FTP/SFTP
client". xsaber is not an FTP client: the main protocol is xsync, SFTP is the
fallback, and FTP/FTPS are out of scope for v1 (see "Non-goals").

---

## Decisions taken

| Decision | Choice | Consequence |
|---|---|---|
| UI framework | **GPUI via GPUI Kit** (`gpui-kit = "0.6"`, longbridge, Apache-2.0), which pulls `gpui-pre 0.3.x` from crates.io. Not Tauri, not a webview. | The user's "GPUIX" resolved to three unrelated projects (see "Research"). The pure-Rust route was chosen. The app never lists `gpui` directly; `use gpui_kit::*` is GPUI. Verified: `cargo build -p example-dock` in the kit repo builds clean on macOS in 3m09s with stable Rust. |
| Widget layer | GPUI Kit components (`Table`, `DockArea`, `h_resizable`/`v_resizable`, `Sheet`, `Dialog`, `Popover`, `PopupMenu`, native menus, `Tree`, `Scrollbar`, `Progress`, `TitleBar`, `WindowBorder`, `StatusBar`, `Switch`, `Select`, `Input`, `Notification`). | Bare `gpui` has 12 primitive elements and no widgets (a text input is ~780 lines in its examples). The kit's gaps are a terminal, a lazy file tree, and cross-table drag; those are built here. |
| SSH library | **russh `=0.63.2`** (Apache-2.0, tokio, MSRV 1.89), `russh-sftp =2.4.0`, `russh-config 0.58`. Crypto backend `ring` (`default-features = false, features = ["ring","rsa","flate2"]`) to avoid `aws-lc-rs` toolchain friction on Windows. | Same pins excalibur already carries, so the two GUIs never disagree on the SSH stack. `russh-keys` is dead; everything is `russh::keys`. One `Arc<client::Handle>` per session carries the PTY, the SFTP subsystem and the `xs --server` exec channel. |
| Primary protocol | **xsync v3** through `xsync-client` (path dep `../xsync/crates/xsync-client`). Session opened as `channel.exec("xs --server <root>")` then `Client::from_stream(channel.into_stream(), features)`. | No change to xsync is needed for browse; `Client::from_stream` is documented for exactly this. `xs` must exist on the remote; if it does not, the session degrades to SFTP and the protocol badge says so. The suite's own `ssh`-spawning `connect_ssh` is not used. |
| Transfer engine | Own worker pool in `xsaber-engine` over the **async v3 file API** (`Mount::open`, `OpenFile::read_verified`, `Stage`/`commit` for uploads) and `russh-sftp` `File` for SFTP. Not `xsync_core::server::run_client_push`. | `run_client_push` is blocking, needs its own second SSH process, emits single-lane progress and never populates `ProgressWatch` on the remote path. The v3 file API gives per-request progress, resume tokens and BLAKE3 verification on commit, and shares the one russh session. |
| Terminal emulator | **`alacritty_terminal =0.26.0`** (Apache-2.0) fed directly from the SSH channel through `vte::ansi::Processor::advance`; never its `tty` module. Rendered by a custom GPUI element. | Same shape Zed uses for its SSH-backed terminals. Zed maintains a fork; budget for pinning and occasional patching. `vt100`, `termwiz`/`wezterm-term` and `avt` were rejected (unmaintained, unpublished, or no `&[u8]` input). |
| Engine / shell split | `crates/engine` (no GPUI dependency) + `crates/term` (emulator model, no GPUI) + `crates/cli` (`xsb`, headless) + `crates/app` (GPUI shell). Engine owns a multi-thread tokio runtime on its own thread; UI receives events over channels and applies them with `cx.spawn`. | Same discipline as excalibur/xebra: rules live in the engine, the shell is thin, the CLI proves the engine headlessly. A current-thread runtime is forbidden in the engine (xsync's blocking bridges deadlock on it). |
| Relationship to excalibur's engine | **Port, do not path-depend.** Copy `error.rs` (`Surface` taxonomy), `hostkeys/`, `secrets/`, and the transfer-queue shape into `xsaber-engine` with attribution; do not add `../excalibur/crates/engine` as a dependency. | excalibur-engine is unversioned, unpublished, carries four xsync path deps and an optional GPL-tainting SMB backend. xsaber is the suite's third GUI, which is the roadmap's stated trigger for a shared crate; that extraction is a suite decision (E17-S3), not something to do implicitly. |
| Credentials | OS keychain via `keyring`, secrets in `Zeroizing` wrappers, never in argv, logs, `Debug`, events or persisted queues. Host keys in an app-private OpenSSH-format file with TOFU states Trusted / Unknown / Changed / Revoked; Changed is never click-through. | Inherits `../AGENTS.md` invariants. The design's amber "NEW HOST KEY … trust and continue?" maps to Unknown only. |
| Fonts | Bundle Barlow 400/500/600/700 and IBM Plex Mono 400 (both OFL) through GPUI's `AssetSource`; no network fonts. Monospace appears only in the terminal. | Matches the design's two-family rule and the desktop packaging constraint. |
| Toolchain | `rust-toolchain.toml` pinned to **1.98.0** (suite rule until Phase 3), edition 2024. | Satisfies russh (1.89), gpui-kit (edition 2024) and alacritty_terminal (1.85). |
| Platforms | macOS (Metal), Linux X11/Wayland (Vulkan), Windows (DirectX) all built in CI from M0; only macOS and Linux are release targets for v1. Windows must stay green in CI but is not packaged, signed or keychain-verified until E16-S4. | The suite calls Windows "the single largest gap"; xsaber does not pretend to close it in v1. |
| Non-goals (v1) | FTP/FTPS, S3, WebDAV, SMB, NFS; tunnels/port forwarding; sync tab beyond a placeholder; auto-update; mobile; a plugin system. | The mock's site tree shows FTP/FTPS/S3 entries. They are rendered as disabled protocol choices in the site manager with a tooltip, recorded in `docs/design-gaps.md`. |

---

## Research (2026-09-08)

What was looked up before writing this plan, so nobody re-derives it.

**"GPUIX".** Three unrelated things: `remorses/gpuix` (gpuix.dev, React/Node
bindings to GPUI; no Rust app API, no custom fonts, no table/tabs/splitter,
no terminal); `AzureZee/gpuix` (a 0-star mirror of Zed's gpui crates plus a
15-line re-export shim, last pushed 2026-05-31); and the crates.io name
`gpuix` 0.1.0, a 785-byte reservation by longbridge that docs.rs labels "not a
library". The library that actually ships the widget set is **GPUI Kit**
(`longbridge/gpui-kit`, renamed from `gpui-component` on 2026-09-03, crate
`gpui-kit` 0.6.0, workspace at commit `dbdc27a` 2026-09-08). Its examples live
in `examples/` (`dock`, `sidebar`, `table_in_scrollable`, `input`, `tiles`,
`dialog_overlay`, `root_borderless`, `window_title`, …); `cargo run -p
example-dock` is the closest to xsaber and `cargo run` alone opens the
70-component gallery. Themes are JSON (`themes/*.json`, 21 bundled) with
`font_family`, `mono_font_family`, `font_size`, `radius` and semantic colour
tokens (`crates/component/src/theme/schema.rs`).

**russh 0.63.2.** `client::Handler::check_server_key(&PublicKeyOrCertificate)`
is called only during the initial key exchange, so it may await a UI prompt.
`AuthResult` is an enum with `remaining_methods` and `partial_success`;
keyboard-interactive is a two-call `_start`/`_respond` loop; agent auth goes
through `authenticate_publickey_with` with `AgentClient` as the signer;
`PrivateKeyWithHashAlg::new` is infallible and RSA needs
`best_supported_rsa_hash`. `Handle` is not `Clone`, dropping it kills the
session, `authenticate_*` take `&mut self` so authenticate before wrapping in
`Arc`. An undrained channel back-pressures the shared TCP stream, so the PTY
must be drained even when the drawer is hidden. `Config::nodelay` defaults to
false (set true); `Handler::adjust_window` is a no-op on clients (issue #770),
so throughput is tuned through `Config::window_size` up front. `russh-sftp`
pipelines writes (`max_concurrent_writes`, default 8) but **not reads**; fast
downloads need concurrent ranged reads over `RawSftpSession`. It does not
implement `posix-rename@openssh.com`, so SFTP rename-over-existing is not
atomic. `known_hosts` helpers: `check_known_hosts_path`, `learn_known_hosts_path`.

**xsync.** `xsync-client` (`crates/xsync-client/src/lib.rs`, 1809 lines, tokio,
no stubs) exposes `Client::from_stream`/`from_split`, `Mount` with `stat`,
`statfs`, `open`, `read_dir_path` (returns `Ok(None)` on overflow, fall back to
the handle path), `read_path`, `read_tree`, `rename(NoReplace|Replace|Exchange)`,
`unlink`, `rmdir`, `mkdir` (one level), `symlink`, `link`, `chown`,
`set_times`, `set_size`, `set_permissions`, `stage`; `OpenFile` with `read`,
`read_verified` (BLAKE3), `write`, `write_if_unchanged`, `read_dir(_all)`;
`Stage` with `resume_token`, `write`, `ranges`, `commit(digest, cookie, mtime)`,
`abort`. Absent from the client: cancel, notify/watch, recursive delete,
`mkdir -p`, owner/group names, per-segment progress on remote transfers. The
server grants exactly `STAGE_RESUME | PATH_LISTING | SUBTREE_LISTING |
PATH_READ`; an unknown message type kills the session, so feature gating is
mandatory. Version negotiation is capabilities-only. `xs --server <root>` is a
hidden flag; mount with the empty export name.

**excalibur.** `crates/engine` is genuinely Tauri-free and already links
`russh =0.63.2` + `russh-sftp =2.4.0` for its SFTP backend, verifies host keys
inside the handshake (`TrustGate::check_server_key`), and carries a
`ShareBackend` trait, `Surface`-classified `EngineError`, keyring secrets, an
app-private known_hosts store, and a broadcast-based transfer queue. It has no
PTY, no bookmarks or settings in Rust (those are TypeScript), no conflict
policy wired into the engine, and no bandwidth limiter.

---

## Repository layout

```
xsaber/
├── Cargo.toml                      workspace: crates/*, examples/*
├── rust-toolchain.toml             1.98.0
├── deny.toml                       cargo-deny (Apache/MIT/OFL allow-list, no GPL)
├── README.md  backlog.md  integration.md   the three live docs
├── plan.md                         this file
├── design_handoff_xsaber_ftp_client/
├── docs/
│   ├── design-gaps.md              every deviation from the mock
│   ├── decisions/0001-gpui-kit.md  ADRs, four digits
│   └── supply-chain.md
├── assets/fonts/                   Barlow-{Regular,Medium,SemiBold,Bold}.ttf, IBMPlexMono-Regular.ttf + OFL.txt
├── assets/icons/                   SVG icon set (Lucide subset) + placeholder blade mark
├── assets/themes/xsaber-dark.json
├── crates/
│   ├── engine/    xsaber-engine    sessions, backends, transfers, sites, settings, secrets, hostkeys (no GPUI)
│   ├── term/      xsaber-term      terminal model: alacritty_terminal + key encoding (no GPUI)
│   ├── cli/       xsb              headless driver of the engine
│   └── app/       xsaber           GPUI shell
└── examples/
    └── kitchen-sink/               every xsaber UI component with mock data, no network (E1-S6)
```

Path dependencies from `crates/engine/Cargo.toml`:
`xsync-client = { path = "../../../xsync/crates/xsync-client" }`. The crate
builds only inside the superproject, like excalibur; `README.md` says so.

---

## Progress

| Milestone | Story | Status |
|---|---|---|
| M0 | E0-S1 … E0-S6, E1-S1 … E1-S6 | E0 complete; E1-S1 and E1-S2 are in progress |
| M1 | E2, E3, E4, E5, E8, E9, E11-S1 … S3 | Not started |
| M2 | E6, E10-S1 … S3 | Not started |
| M3 | E7 | Not started |
| M4 | E12, E13, E11-S4, E10-S4 | Not started |
| M5 | E14, E15, E16, E17 | Not started |

E0 is complete: xsaber is published at `https://github.com/s4njee/xsaber` on
the `xcalibur` branch at `85876b3`, and the clean recursive suite checkout
`/tmp/xsaber-e0-clean.83Ddis` built and passed its 4 engine tests with Rust
1.98.0. GitHub run
`https://github.com/s4njee/xsaber/actions/runs/34194549604` passed the macOS and
Ubuntu Rust, supply-chain, and release app jobs; Windows reported the known
`xsync-core` `rustix::fs` failure allowed by E0 until E16-S4. Root registration
and the superproject pin remain E17-S1 work. E1-S1 and E1-S2 are in progress:
`9ccd066` provides the theme asset, typed tokens, and a `ThemeSet` JSON
round-trip test; `80d4cce` provides the licensed local font assets and
provenance. Theme-registry startup loading, font `AssetSource` registration,
and the fallback-independent rendering proof remain open.

---

## Epics and stories

Story ids are `E<n>-S<m>`. `-S0` marks a prerequisite owned by another
project. Each story carries **AC** bullets that are falsifiable; the last
bullet names the test obligation.

### E0 — Project scaffold and suite membership

Turn `xsaber/` into a repository the suite can pin.

#### E0-S1 — Repository and workspace
Initialise git in `xsaber/`, create the workspace above with empty crates that
compile, pin the toolchain, add `.gitignore`, `LICENSE` (Apache-2.0, matching
russh/gpui/alacritty_terminal), and `deny.toml`. Publish the `xcalibur` branch
so the suite can consume a reproducible commit; root registration and pinning
are deliberately owned by E17-S1.
**AC**
- `cargo build --workspace` and `cargo test --workspace` pass on a clean checkout inside the superproject.
- `rust-toolchain.toml` says `1.98.0`; `cargo --version` under the pin is what CI uses.
- `cargo deny check` passes with GPL and AGPL denied and OFL allowed for the fonts.
- The published remote `https://github.com/s4njee/xsaber` has an `xcalibur` branch containing the checked workspace; the published commit and clean-checkout result are recorded in `backlog.md`.
- E0-S1 does **not** add a root `.gitmodules` entry or move the superproject pin. E17-S1 registers this published commit and proves six clean pins.

#### E0-S2 — CI
`.github/workflows/ci.yml` modelled on excalibur's: `fmt --check`, `clippy
--workspace --all-targets -D warnings`, `test --workspace` on macOS, Linux and
Windows; `cargo deny check` and `cargo audit`; a `package` job that compiles
the app in release mode. This is an E0 compilation gate, not native bundle
creation or verification; actual unsigned/native bundles belong to E16. Linux
runner installs the GPUI system deps
(fontconfig, wayland, xkbcommon-x11, libvulkan, cmake, clang).
**AC**
- The workflow checks out `s4njee/xcalibur` with `submodules: recursive` so the xsync path dependency resolves (excalibur's CI omits this and cannot build its own xsync backend; do not inherit that).
- Windows is `continue-on-error: true` until E16-S4 and is reported, not hidden.
- A PR that adds a `clippy::unwrap_used` in `crates/engine` fails the `rust` job (the engine denies `unwrap_used`, `expect_used`, `panic`, matching russh's own policy).
- The package job runs the app's release-mode compilation (for example, `cargo build --release --manifest-path crates/app/Cargo.toml`); it does not claim to produce or verify a native bundle. E16 owns bundle tooling, assets, signing and platform launch checks.

#### E0-S3 — The three live docs
`README.md` (what it is, how it fits the suite, build/run/test, the `plan.md`
exception), `backlog.md` (status snapshot bullets: Milestone, Branch, Last 5
commits, Test gate, Not gated), `integration.md` (the xsync seam, the
excalibur port, the shared-crate question).
**AC**
- `../README.md` and `../AGENTS.md` gain an xsaber row and an entry in the layout tree; `../AGENTS.md`'s per-project table gets a build/test row.
- `integration.md` lists every xsync prerequisite story from E17-S2 by id.

#### E0-S4 — ADRs
`docs/decisions/0001-gpui-kit.md` (why not Tauri, why not remorses/gpuix or
bare gpui), `0002-xsync-primary-sftp-fallback.md`, `0003-alacritty-terminal.md`,
`0004-port-not-depend-on-excalibur-engine.md`, `0005-ring-not-aws-lc.md`.
**AC**
- Each ADR has date, status, context, decision, consequences, and the alternatives rejected with the evidence from "Research".

#### E0-S5 — Design gaps ledger
`docs/design-gaps.md` seeded with the deviations already known: protocol
badge reads `XSYNC · ED25519` (or `SFTP · ED25519` on fallback); the site
tree's FTP/FTPS/S3 entries become disabled protocol options; "Preserve
timestamps" help text loses "MFMT"; "Verify checksums" compares BLAKE3 (xsync)
not SHA-256; the Apply button stays solid pending the mock's own open question;
the Sync tab is a placeholder in v1.
**AC**
- Every later story that departs from the mock appends a dated row here; CI greps that the file exists and is non-empty.

#### E0-S6 — Logging and diagnostics
`tracing` with a rolling file under the app data dir plus the in-app message
log (E10-S3) as a subscriber. Secrets are redacted at the `Secret` type, not
by filters.
**AC**
- A `Secret` in any span field prints `<redacted>`; a test asserts this for password, passphrase and key material.
- `XSABER_LOG=debug` raises verbosity without a rebuild.

### E1 — Design system in GPUI Kit

Encode the handoff's tokens once so every screen inherits them.

#### E1-S1 — Theme
`assets/themes/xsaber-dark.json` mapping the design tokens onto the kit's
schema: surfaces (`#0c0d10` bg, `#0a0b0e` sunken, `#101216` strip, `#14161b`
surface, `#1a1d24` raised), lines (`#1e222a`, `#24282f`, `#2e333c`), text
ramp (`#e6e8ec` … `#4b525e`), accent `#ff6b3d` / `#ff8b5f` / ink `#1a0d07`,
success `#3ecfb2`, warning `#ffb020`, danger `#ff4d4f`/`#ff7a7c`, info
`#7aa2ff`/`#62b6ff`, violet `#b39bff`, amber `#ffc740`; radius 5px controls,
10px window. Tokens the kit schema lacks live in a `xsaber::tokens` module
(the ext-badge palette, row heights, the 9.5px letter-spaced label style).
**AC**
- The theme loads through the kit's theme registry at startup; a test round-trips the JSON through `ThemeConfig` without unknown-field warnings.
- No hex literal appears in `crates/app` outside `tokens.rs` (a CI grep enforces it).

#### E1-S2 — Fonts
Bundle Barlow 400/500/600/700 and IBM Plex Mono 400 via an `AssetSource`
implementation and `cx.text_system().add_fonts`. Theme sets `font_family =
"Barlow"`, `mono_font_family = "IBM Plex Mono"`.
**AC**
- The app renders with Barlow on a machine with neither font installed (CI's Linux runner is such a machine; a visual test asserts glyph advance widths differ from the fallback).
- `assets/fonts/OFL.txt` ships in every bundle (E16).

**Blocker:** The bundle-shipping AC waits on `E16`; the remaining `AssetSource`,
font-registration, and fallback-width work is local to this story and is not
externally blocked.

#### E1-S3 — Icons and wordmark
Vendor a Lucide subset as SVG (refresh, upload, download, sync, filter,
terminal, search, lock, chevrons, plus, close, folder) and the placeholder
blade mark. Expose `xsaber::icons::Icon` over the kit's `Icon` component.
**AC**
- Every glyph in the mock's toolbar, drawer and status bar has an icon; none is a raster.
- Icon colour follows `currentColor` so the toggled Terminal button's `#ff8b5f` stroke works from the theme.

#### E1-S4 — Density primitives
Row-height and text-style constants for the design's scale (25/26/27/28/30/32/34/36/40/46px; 8.5–15px Barlow scale; letter-spaced uppercase labels under 11px), and thin wrappers: `Chip`, `ExtBadge`, `StateBadge`, `SegmentedField` (the HOST/USER/PORT strip), `OutlinedAccentButton` (the Connect treatment), `SectionLabel`.
**AC**
- `Table` rows can be forced to 25px (verify the kit's row-height API; if it is fixed per size variant, the file table is built on `virtual_list` instead and this story records that in `design-gaps.md`).
- The kitchen-sink example (E1-S6) shows each primitive in every state named in the handoff (default, hover, selected, focused, disabled).

**Blocker:** The all-state visual proof cannot close until the `E1-S6`
kitchen-sink fixture exists; implementation and row-height API research can
proceed independently.

#### E1-S5 — Window chrome and menus
Custom title bar via the kit's `TitleBar`/`WindowBorder`: wordmark, in-window
menu bar (File · Edit · View · Transfer · Server · Bookmarks · Help), spacer,
protocol badge, platform window controls (traffic lights on macOS, right-side
controls on Windows/Linux). Mirror the menu into the macOS system menu with
`cx.set_menus`.
**AC**
- On macOS the in-window menu bar and the system menu dispatch the same `actions!`; on Linux/Windows only the in-window bar exists.
- Window drag, double-click-to-zoom and the 10px corner radius work on all three platforms (Wayland uses client-side decorations).

#### E1-S6 — Kitchen-sink example app
`examples/kitchen-sink`: every xsaber component rendered with the mock's sample
data and no network. This is the repo's example app and the visual-regression
fixture.
**AC**
- `cargo run -p kitchen-sink` opens a window reproducing screens `1a`–`1e` as static views selectable from a sidebar.
- Screenshots of each screen are captured under `test-support` and diffed in E15-S3.

**Blocker:** Screenshot-diff acceptance depends on the `E15-S3` visual
regression harness; the static example itself is not blocked by that harness.

### E2 — Engine: SSH sessions (russh)

One russh session per connected host, shared by everything.

#### E2-S1 — Session actor
`Session` owns `Arc<client::Handle<Client>>` after authentication, a tokio
task supervising it, `Config { nodelay: true, window_size: 8 MiB,
keepalive_interval: 30s, keepalive_max: 3, inactivity_timeout: None }`, and
an `Event` stream (`Connecting(step)`, `HostKey(prompt)`, `Authenticated`,
`Degraded`, `Disconnected(reason)`).
**AC**
- `channel_open_session` can be called concurrently from three tasks (PTY, SFTP, exec) and all three succeed on one TCP connection; an integration test does exactly this against the in-process test server (E15-S1).
- Dropping the last `Session` reference closes the connection; no channel outlives it.
- The handshake log lines in screen `1c` (`resolved host`, `TCP established`, `Server: …`, `key exchange …`, `Verifying host key …`, `Authenticating …`) are emitted as `Connecting(step)` events with elapsed times.

**Blocker:** The live actor/channel-lifecycle proof requires the `E15-S1`
in-process SSH server. `cc3a57e` provides the config, bounded events, elapsed
handshake steps, and lifecycle state it will exercise.

#### E2-S2 — Host-key trust
Port excalibur's `hostkeys` module: app-private OpenSSH-format file, TOFU
states Trusted / Unknown / Changed / Revoked, atomic writes, hashed and
`@cert-authority` lines skipped. `Client::check_server_key` consults it and,
on Unknown, sends a `HostKey` prompt carrying `SHA256:` fingerprint and
algorithm, then awaits Trust / Trust once / Reject over a oneshot.
**AC**
- Changed keys never prompt; they fail with a `Surface::Banner` error carrying the known_hosts line.
- Trust once does not write the file; Trust does; Reject yields `russh::Error::UnknownKey` mapped to `AuthenticationFailed`.
- Host certificates (`PublicKeyOrCertificate::Certificate`) are verified against `@cert-authority` entries when present and otherwise treated as Unknown with the CA fingerprint shown.

**Blocker:** `cc3a57e` supplies app-private TOFU, changed/revoked policy, and
atomic persistence, but the russh callback and certificate-authority verifier
are not implemented. `@cert-authority` entries are safely skipped until that
verification path exists.

#### E2-S3 — Authentication ladder
Order: agent (`AgentClient::connect_env`, Pageant/named pipe on Windows) →
configured key file (`load_secret_key`, `KeyIsEncrypted` → passphrase prompt,
`best_supported_rsa_hash` for RSA) → password → keyboard-interactive loop.
Driven by `AuthResult::Failure { remaining_methods, partial_success }`.
**AC**
- Every prompt (passphrase, password, KI questions) is an engine event answered by the UI; the engine never reads stdin or env for a secret.
- A server requiring publickey **and** KI (partial_success) authenticates.
- Secrets pass through `Secret` and are zeroized after use; the test from E0-S6 covers this path.

**Blocker:** `dd069e8` provides the secret-safe ladder and partial-success
policy, but proving real publickey-plus-KI authentication requires the
`E15-S1` SSH fixture and russh adapter calls.

#### E2-S4 — `~/.ssh/config`
`russh-config::parse_home` for `HostName`, `User`, `Port`, `IdentityFile`,
`ProxyCommand` (via `connect_stream` on `Stream::proxy_command`). `ProxyJump`
is parsed and reported as unsupported in v1 (`Match` blocks likewise; issue
#699).
**AC**
- Quick-connecting `sftp://alias` where `alias` is a `Host` block resolves host/user/port/key without the user typing them.
- A `ProxyJump` entry produces a `Surface::Field` error naming the limitation rather than a silent TCP connect.

**Blocker:** `dd069e8` provides a side-effect-free resolver and explicit
ProxyJump/Match field errors. Full completion waits on `russh-config::parse_home`
and `ProxyCommand` stream wiring; the required crate lookup was blocked by a
transient crates.io DNS failure during this tranche.

#### E2-S5 — Reconnect supervisor
On `Handler::disconnected` or `Handle::is_closed`, mark the session Degraded
(amber dot), attempt reconnection with backoff (1s, 2s, 4s, cap 30s, stop after
5), re-authenticate silently only with non-interactive methods, re-open the
PTY and SFTP channels, and re-mount xsync. Transfers in flight resume via
their stage tokens (E6-S5).
**AC**
- Killing the test server mid-listing yields a Degraded tab, then Connected again within 10s once the server returns, without a user prompt.
- A password-only site does not silently retry; it surfaces `Surface::Banner` with a Reconnect action.

### E3 — Engine: xsync v3 remote filesystem

#### E3-S0 — Prerequisites in `../xsync`
Tracked in E17-S2. Nothing here blocks M1; everything here is a quality gap.

#### E3-S1 — `RemoteFs` trait
Object-safe, `async_trait`, `Send + Sync`: `caps`, `statfs`, `read_dir(path)
-> Vec<Entry>`, `stat`, `lstat`, `open_read(path) -> Box<dyn ReadAt>`,
`open_write(path, size_hint, mode) -> Box<dyn WriteStage>`, `mkdir`,
`mkdir_all` (client-side loop), `rename(src, dst, mode)`, `remove(path,
recursive)` (client-side walk), `set_permissions`, `set_times`, `symlink`,
`read_link`, `ping`. `Entry` carries `Attrs { size, mode, uid, gid, mtime,
kind, owner_names: Option }` and a `FileClass` for the badge palette.
**AC**
- Excalibur's `ShareBackend` methods are a subset (so a later shared crate can host both); the difference (`set_permissions`, `set_times`, `symlink`, `read_link`, `lstat`) is listed in `integration.md`.
- `Caps` is a bitflags set: `RANDOM_ACCESS | RESUME_WRITE | ATOMIC_RENAME | VERIFIED_COMMIT | SYMLINKS | OWNER_NAMES | WATCH`.

#### E3-S2 — xsync backend over an exec channel
`XsyncFs::open(session, root)`: `channel.exec(true, "PATH=\"$HOME/.local/bin:$PATH\" 'xs' '--server' '<root>'")` with the root shell-quoted exactly as `xsync-client::connect_ssh` does; `Client::from_stream(channel.into_stream(), STAGE_RESUME | PATH_LISTING | SUBTREE_LISTING | PATH_READ)`; `mount(b"", Access::ReadWrite)`; `keepalive_every(20s)`.
**AC**
- All `RemoteFs` methods are implemented by the `Mount`/`OpenFile`/`Stage` calls named in "Research", with `read_dir` falling back to the handle path when `read_dir_path` returns `Ok(None)`.
- A remote without `xs` on `PATH` is detected from the exec channel's exit status and stderr within 3s and returns `Error::XsyncUnavailable` (not a hang), which E4-S3 turns into the SFTP fallback.
- `remove(recursive=true)` deletes a 3-level tree in the test server; `mkdir_all` creates `a/b/c`.
- Cancel: closing the exec channel aborts outstanding requests without wedging the session's other channels (test with a 100 MB read cancelled at 10%).
- Integration tests run against a real `xs --server` spawned locally over the in-process SSH server (E15-S1) and, in a nightly job, over Docker `openssh-server` with `xs` installed.

#### E3-S3 — Capability and protocol gating
Advertise nothing the server did not grant; never send a type outside the
granted feature set (an unknown type kills the session).
**AC**
- `caps()` reflects `MountInfo.supports` and the granted features; `OWNER_NAMES` is off until xsync grants it (E17-S2).
- A mock server granting zero features still browses (handle path only) and reports `RESUME_WRITE` off.

#### E3-S4 — Remote binary override and version display
Site setting `Remote xs path` (default `xs`) and a `Server: xsync <caps>` line
in the handshake log. Bootstrap (copying an `xs` binary to the host) is
deferred; the client crate has no such path.
**AC**
- A site with `/opt/xsync/bin/xs` connects; the setting is structured (an argv element), never string-interpolated into a shell.

### E4 — Engine: SFTP fallback (russh-sftp)

#### E4-S1 — SFTP backend
`SftpFs` over `channel.request_subsystem(true, "sftp")` and
`SftpSession::new_with_config(channel.into_stream(), Config { max_packet_len:
262144, max_concurrent_writes: 32, request_timeout_secs: 30 })`, keeping a
`RawSftpSession` for `stat`/`lstat`/`setstat`/`statvfs`/`fsync` and pipelined
reads (8 outstanding `SSH_FXP_READ`s per file).
**AC**
- Download of a 256 MiB file over a 50 ms-RTT netem link reaches ≥ 70% of the xsync backend's throughput (the un-pipelined `File::poll_read` path would be ~5×  slower; the test records both numbers).
- `rename` over an existing destination with `RenameMode::Replace` removes-then-renames and reports `ATOMIC_RENAME` off; with `NoReplace` it fails cleanly.
- `VERIFIED_COMMIT` is off; the UI's "verified" tick is never shown for SFTP.

#### E4-S2 — Shared trust and auth
The SFTP backend reuses E2's session; no second handshake, no second prompt.
**AC**
- Connecting to a host with and without `xs` shows exactly one host-key prompt in both cases.

#### E4-S3 — Protocol selection
`Site.protocol ∈ { Auto, Xsync, Sftp }`, default Auto: try xsync, fall back to
SFTP on `XsyncUnavailable`, and record which one was chosen on the session.
**AC**
- The protocol badge and the session-tab dot reflect the choice: teal `XSYNC · <keyalg>`, amber `SFTP · <keyalg>` with tooltip "xs not found on host; using SFTP", per `design-gaps.md`.
- Forcing `Sftp` on a host with `xs` uses SFTP.

### E5 — Engine: local filesystem and path model

#### E5-S1 — Local backend
`LocalFs` implementing `RemoteFs` with `spawn_blocking`, `trash` for delete
(with a permanent-delete option), reflink-aware copy where available,
`#[cfg(windows)]` arms for drive roots and `\\?\` paths.
**AC**
- `read_dir` of a 50k-entry directory returns in < 300 ms on the CI runner.
- Windows drive letters appear as roots; `..` from `C:\` is a no-op.

#### E5-S2 — Path model
`RemotePath` (POSIX, from the server) and `LocalPath` (`std::path`), each with
breadcrumb splitting, join, parent, display truncation with middle ellipsis,
and validation against traversal and target-platform name collisions
(`../AGENTS.md` invariants).
**AC**
- Property tests: `join(parent(p), name(p)) == p`; no input yields a path escaping the site root.

### E6 — Engine: transfer queue

#### E6-S1 — Queue model
`Transfer { id, direction, src, dst, bytes_total, bytes_done, speed, eta,
state, error, retries, verified, started, finished }`, `State ∈ { Queued,
Transferring, Paused, Complete, Failed, Skipped, Cancelled }`. One row per
file (folders are expanded by a planner task into rows grouped under a batch
id). Events over a broadcast channel carrying the whole row, coalesced to
250 ms per row plus a 1 s tick for aggregates (count, active, ↑/↓ throughput,
latency).
**AC**
- 10k queued rows render and update without dropping frames in the kitchen-sink stress fixture (E15-S4 gate).
- Aggregates are computed in the engine; the status bar and the `N QUEUED` chip read the same struct.

#### E6-S2 — Workers and concurrency
`N` workers from Preferences → Simultaneous transfers (default 3), per-session
cap, xsync's `MAX_CONCURRENT_CONNECTIONS`-style limit of 8 channels per host.
Runtime changes lower the ceiling without killing running rows.
**AC**
- With `N = 3`, exactly three rows are `Transferring` at once in the test; changing to 1 lets the two extra finish, then holds at one.

#### E6-S3 — Upload path (xsync)
`Stage(dst, size, mode, resume_token)` → chunked `write` (8 MiB) → `commit(blake3, expect_cookie, mtime)`. Preserve-timestamps preference sets `mtime`. Verified tick from commit.
**AC**
- A 1 GiB upload interrupted at 40% resumes from the staged ranges after reconnect (E2-S5) and commits with a matching digest; the row shows `verified`.
- A digest mismatch leaves no partial file at `dst` and the row shows `Failed: integrity`.

#### E6-S4 — Download path (xsync and SFTP)
`open_read` + `read_verified` (xsync) or pipelined ranged reads (SFTP) into a
`.xsaber-part` file next to the destination, then an atomic rename into place;
on xsync, BLAKE3 per read. Optional parallel parts for files > 64 MiB
(Preferences → Split large files) using multiple ranged readers.
**AC**
- Parts are only ever created under a name xsaber owns and are removed on cancel; an unrelated `*.part` file in the folder is untouched.
- Splitting a 512 MiB file into 4 parts on a LAN test server is ≥ 1.5× faster than one stream over SFTP and not slower over xsync.

#### E6-S5 — Conflict policy
`When the file exists` ∈ { Overwrite if newer (default), Overwrite, Skip,
Rename (keep both), Resume, Ask }. Decided by the engine at execution time
against a fresh `stat` (not at planning time). `Ask` raises a `Conflict` event
the UI answers per row or "apply to all".
**AC**
- Each policy has an integration test with a pre-existing destination; `Rename` produces `name (2).ext`; `Resume` only when `RESUME_WRITE` and the existing size is smaller than the source.

#### E6-S6 — Filters and dotfile guard
Ignore patterns (gitignore syntax via `ignore`/`globset`, default `.DS_Store,
.git/, *.log`), applied in both directions at planning time; "Warn before
uploading dotfiles" raises a batch-level confirmation listing the files with
`.env*` highlighted as secrets.
**AC**
- The mock's queue shows `.env.production` as `SKIPPED`; the test reproduces that from the default settings.

#### E6-S7 — Retry, cancel, pause
Failed rows stay in the queue with a Retry affordance (auto-retry ×2 with
backoff for `transient()` errors); Cancel drains to a resumable stage; Pause
per row and global.
**AC**
- Cancelling a transfer never removes an existing destination file.
- Retry of a `Failed` row reuses its stage token.

#### E6-S8 — Bandwidth limit
Token-bucket limiter shared by all sessions (Preferences → Speed limit,
upload and download separately, `unlimited` default), applied at the chunk
scheduler so segments do not tear.
**AC**
- With a 1 MB/s limit, a 20 s transfer measures 0.9–1.1 MB/s; changing the limit mid-transfer takes effect within one chunk.

#### E6-S9 — History and persistence
Completed/failed/skipped rows move to a per-day history (`history/YYYY-MM-DD.jsonl`); the live queue persists to `transfers.json` atomically with volatile fields blanked; both under the app data dir. Restart restores Queued and Paused rows and marks interrupted `Transferring` rows Paused.
**AC**
- The `HISTORY · TODAY` header's `18 transfers · 412.6 MB · 1 failed` summary is derived from the day file.
- No persisted row contains a secret or an absolute path outside the site's configured roots being logged in plain text beyond what the row needs.

### E7 — Terminal drawer

#### E7-S1 — Terminal model crate
`xsaber-term`: `Terminal { term: alacritty_terminal::Term<Listener>, parser:
vte::ansi::Processor }` with `advance(&[u8])`, `resize(cols, rows)`,
`renderable()`, `damage()`, scrollback (10k lines), selection
(`selection_to_string`), `Event::PtyWrite` routed back to the channel, and
key/mouse encoding respecting application-cursor, bracketed-paste and mouse
modes. No GPUI dependency.
**AC**
- The vttest-style fixture corpus (cursor movement, SGR colours, alt screen, bracketed paste, DA1 reply) round-trips; `PtyWrite` replies match a reference transcript.
- Keystroke encoding tests for arrows, Home/End, function keys, Ctrl+letters, Alt+letters, Shift+Tab in both normal and application modes.

#### E7-S2 — PTY channel
Per session: `channel_open_session`, `request_pty(true, "xterm-256color", cols, rows, 0, 0, &[(ECHO,1),(ICANON,1),(ISIG,1),(IUTF8,1),(TTY_OP_ISPEED,38400),(TTY_OP_OSPEED,38400)])`, `request_shell`, `split()` into a reader task feeding `Terminal::advance` and a writer behind an `Arc`. `window_change` on resize. Exit status ends the tab with a "shell exited (code)" line and a Restart affordance.
**AC**
- The reader task runs regardless of drawer visibility; a hidden drawer with 50 MB of output does not stall an SFTP download on the same session (integration test).
- Switching drawer tabs does not close the channel; collapsing the drawer does not either.
- `TERM`, `LANG` are sent via `set_env` where the server allows it; failures are ignored.

#### E7-S3 — Terminal element (GPUI)
A custom element rendering the grid with IBM Plex Mono 11.5px/1.55 using
`TextRun`s per colour run, damage-driven re-layout, a 7×14 block cursor with
the `pulse` animation, 256/true colour, bold/italic/underline/strikethrough,
selection highlight, scrollbar, click-to-focus, IME-safe input via the kit's
focus/keystroke handling, copy on selection (setting) and paste with
bracketed-paste.
**AC**
- 60 fps with `yes | head -c 50M` streaming on the CI Mac (frame time < 16 ms measured with `gpui-fps`).
- Cmd/Ctrl+C with a selection copies; without one it sends `^C`.
- Font, prompt colour `#ff6b3d`, output `#7f8794`, padding `10px 12px` match the mock.

#### E7-S4 — Drawer integration
Drawer tab bar (Terminal · Transfer queue with count badge · Message log ·
Sync), right-side `user@host:cwd · shell` label, `⌃\`` toggles collapse and
focuses the terminal, drag-resize 120px–60% of window height, double-click
tab bar to collapse, state persisted per window.
**AC**
- The shortcut works from any focused control; the terminal keeps focus after a queue update repaints the drawer.

#### E7-S5 — Working-directory sync and "Open in terminal"
Track the shell's cwd from OSC 7 when the remote shell emits it; otherwise from
a lightweight prompt hook the user can opt into (documented, not injected
silently). "Open in terminal" from the remote pane's context menu writes
`cd '<path>'\n` (single-quote escaped) to the PTY and expands the drawer.
**AC**
- The label updates to the new cwd within one prompt cycle on bash and zsh with OSC 7 configured; without it the label shows the last `cd` sent.
- Paths with quotes and spaces are escaped correctly (unit test on the escaper).

### E8 — UI: main window

#### E8-S1 — Band layout
Title bar 40 · session tabs 34 · toolbar 46 · dual pane (flex) · drawer
(resizable, collapsible) · status bar 26, 1px `#24282f` rules between bands,
window background `#0c0d10`.
**AC**
- At 1420×920 the bands measure exactly as the mock; below 900px width the panes switch to a single pane with a Local/Remote segmented selector (design recommendation).

#### E8-S2 — Session tabs
First tab flush with the window edge, 5px status dot (teal connected, grey
idle, amber degraded/SFTP-fallback), active tab `#1a1d24` with a 2px accent
top border, `+` opens Quick connect, middle-click/`⌘W` closes with a
confirmation when transfers are active.
**AC**
- Closing a tab cancels its transfers only after confirmation and never orphans a PTY channel.

#### E8-S3 — Toolbar
Segmented HOST/USER/PORT inputs (real text inputs), outlined Connect, icon
buttons (Refresh, Upload, Download, Synchronized browsing, Filter, Terminal
toggle-on state), right-aligned "Filter files" chip and `N QUEUED` chip.
**AC**
- Upload/Download act on the active pane's selection and enqueue via E6; Terminal toggles E7-S4; the queued chip reads E6 aggregates.

#### E8-S4 — Status bar
Left: `Connected` (teal) and the URL (`xsync://` or `sftp://`); right:
`N queued · M active`, `↑ rate`, `↓ rate`, `latency` (from keepalive RTT).
**AC**
- Rates match the queue aggregates within one tick; latency updates every keepalive.

#### E8-S5 — Actions and keymap
`actions!` for every menu item and shortcut in the handoff (`⌘K`, `⌃\``,
`⌘R` refresh, `⌘↑` parent, Enter open, Delete, `F2` rename, `⌘,`
preferences, `⌘⇧S` site manager), a `keymap.json` users can override, and a
command palette (`⌘⇧P`) over the same actions.
**AC**
- Every menu entry has an action; every action is reachable from the palette; a test enumerates both and fails on a mismatch.

### E9 — UI: file pane

One component, instantiated for Local and Remote.

#### E9-S1 — Pane header and path field
`LOCAL`/`REMOTE` chip, lock glyph on the remote when encrypted, editable path
field with breadcrumb mode on hover and free-text mode on click (Enter
navigates, Esc reverts), right-aligned meta (`14 items · 4 selected · 1.8 MB`
/ `13 items · 3 writing`).
**AC**
- Typing a nonexistent path shows a `Surface::Field` error inline, not a modal.

#### E9-S2 — File table
Virtualized 25px rows, grid `minmax(0,1fr) 76 100 128 96`, 26px column
header with sort on click (name/size/type/modified/perms; directories first;
stable), extension badges from the palette, dimmed rows for `..`, dotfiles,
lock files and superseded builds, row states selected / queued-for-write /
hover, keyboard navigation (arrows, PageUp/Down, Home/End, type-ahead).
**AC**
- 50k rows scroll at 60 fps; sort of 50k rows completes < 50 ms.
- Both panes share the column grid so the two tables align as one ruler (pixel test in the kitchen sink).

#### E9-S3 — Selection and navigation
Click, shift-click range, cmd/ctrl-click toggle, drag-select, `⌘A`; double
click descends, `..` ascends, Backspace ascends; per-path listing cache with
explicit refresh; loading and error states inline.
**AC**
- Selection survives a refresh that keeps the same entries and is dropped for removed ones.

#### E9-S4 — Context menu and verbs
Download/Upload, Open, Open in terminal (E7-S5), Rename (inline editor),
Delete (trash locally; confirm remotely), New folder, Permissions dialog
(chmod with rwx grid and octal field), Copy path / Copy URL, Refresh,
Properties.
**AC**
- Every verb is an action (E8-S5) with the same behaviour from menu, keymap and context menu; destructive verbs confirm and never bypass the engine's validation.

#### E9-S5 — Drag and drop between panes
Dragging a selection to the opposite pane enqueues transfers to that pane's
current directory (or the hovered folder row); dropping OS files onto the
remote pane uploads them; dragging from the remote pane to the OS is deferred.
**AC**
- The drop target highlights the folder row or the pane; a drop with 4 files creates 4 queue rows with the `dist → releases/20260907` route text.

#### E9-S6 — Synchronized browsing and filter
Toggle keeps both paths in step when the trees mirror (descend/ascend applied
to both; divergence disables with a message-log entry). "Filter files" chip
opens a text filter applied to both panes (glob or substring).
**AC**
- Descending into `assets` on the remote descends locally when `assets` exists there, and reports the divergence otherwise.

### E10 — UI: drawer tabs

#### E10-S1 — Transfer queue table
32px rows, grid `minmax(0,1fr) 84 220 84 72 96`, direction glyph
(`↑ #ff8b5f`, `↓ #62b6ff`), filename + route, 4px progress track with
accent/danger/neutral fills, state badges (TRANSFERRING/QUEUED/COMPLETE/
FAILED/SKIPPED), active row wash, right-side `↑ rate · N workers · retry ×2`,
context menu (Retry, Pause, Cancel, Remove, Open folder), drag reorder.
**AC**
- Updates coalesce (E6-S1) so a 500-row queue with 3 active rows repaints only those rows per tick (measured with the kit inspector).

#### E10-S2 — History section
`HISTORY · TODAY` header with the summary, 27px rows (time, path with glyph,
size, duration, badge), "Show earlier days" loads previous day files.
**AC**
- Reads E6-S9 day files; clearing history asks first.

#### E10-S3 — Message log
Chronological engine log (connection steps, errors with `Surface`, verbs,
transfers) with level filter and copy; the same subscriber as E0-S6 minus
debug noise.
**AC**
- A failed transfer appears with its `Surface::TransferRow` reason and a link that selects the queue row.

#### E10-S4 — Sync tab placeholder
A panel explaining that folder sync ships later, with a link to the xsync CLI
for now. Recorded in `design-gaps.md`.
**AC**
- No dead controls.

### E11 — UI: connect flows

#### E11-S1 — Quick connect popover (`⌘K`)
Focused URL field with scheme prefix (`xsync://`, `sftp://`), key-file and
port fields, RECENT list with status dots and relative times, keyboard cursor
row wash, Connect (outlined) and Save site.
**AC**
- `user@host`, `host:port`, and `~/.ssh/config` aliases all parse; recents persist (last 20) with no secrets.

#### E11-S2 — Connecting card
Spinner, 2px progress bar, handshake log with elapsed times and `✓`/`●`/blank
marks from E2-S1 events, Cancel.
**AC**
- Cancel during key exchange closes the socket within 1 s and returns to the popover.

#### E11-S3 — Host-key and auth prompts
`NEW HOST KEY` footer prompt with Trust / Trust once / Reject; passphrase,
password and keyboard-interactive prompts rendered in the same card.
**AC**
- Prompts are keyboard-completable; the answer goes back over the E2 oneshot; Reject shows the fingerprint in the message log.

#### E11-S4 — Success and failure transitions
On success the tab dot turns teal, both panes populate (remote at the site's
remote dir, local at its local dir) and the protocol badge is set. On failure
the error surfaces per `Surface` (banner for connection, field for validation).
**AC**
- No modal alert exists anywhere in the connect flow.

### E12 — UI: site manager

#### E12-S1 — Sites model and persistence
`Site { id, name, group, protocol: Auto|Xsync|Sftp, host, port, user, auth:
Agent|KeyFile{path}|Password|KeyboardInteractive, remote_dir, local_dir,
remote_xs_path, comment, last_used, host_key_fingerprint }`, groups as a
tree, stored at `<data>/xsaber/sites.json` with `#[serde(default)]` on every
field and never-rename-keys migrations; secrets in the keychain keyed by site
id (service `xsaber`).
**AC**
- Deleting a site deletes its keychain entry; duplicating does not copy the secret.
- Round-trip test of the JSON with an unknown future field preserved.

#### E12-S2 — Dialog
900×558: SITES tree (groups with `▾`, children indented 26px, tag badges
XSYNC/SFTP; FTP/FTPS/S3 shown disabled per `design-gaps.md`), New site /
Folder / Duplicate, tabs General · Advanced · Transfer · Charset (Charset holds
only the remote filename encoding; deprioritised), form rows per the mock
including Key file Browse… (native file dialog via `cx.prompt_for_paths`),
status chips (HOST KEY VERIFIED, fingerprint, last used), footer Delete ·
Cancel · Save · Connect.
**AC**
- Connect from the dialog runs E11 with the dialog closed and the site's tab opened.

#### E12-S3 — Advanced and Transfer tabs
Advanced: remote `xs` path, keepalive interval, `~/.ssh/config` use toggle.
Transfer: per-site overrides of concurrency, conflict policy, speed limit,
ignore patterns.
**AC**
- A per-site override wins over Preferences and is shown with a "overrides preferences" hint.

### E13 — UI: preferences

#### E13-S1 — Settings model
`Settings` struct with per-field sanitisation and an exhaustive `match` so a
new field without a validator fails to compile; stored at
`<data>/xsaber/settings.json`; read by the engine at runtime (concurrency,
conflict policy, verification, ignore patterns, speed limits, split threshold).
**AC**
- Corrupt JSON yields defaults plus a message-log warning, never a crash.

#### E13-S2 — Dialog
940×652: left nav (Connection, Transfers, Interface, Terminal, Security, File
editing, Updates), the Transfers section exactly as the mock (CONCURRENCY,
CONFLICTS & INTEGRITY, FILTERS groups with toggles and value fields), Search
settings chip filtering rows, footer Restore defaults · Cancel · Apply.
**AC**
- Apply persists and the engine picks the change up without reconnecting.
- Other sections carry at least: Connection (default port, keepalive, known_hosts path), Interface (theme, density), Terminal (font size, scrollback, copy-on-select), Security (agent use, remember passwords), Updates (placeholder, off).

### E14 — Headless CLI `xsb`

#### E14-S1 — Verbs
`xsb ls|stat|get|put|mkdir|rm|mv|chmod|df|reachable|sites` with `clap`, site
or URL targets, `--json`, host-key `--trust` that cannot override a Changed
key, progress on stderr, data on stdout.
**AC**
- The CLI drives the same `RemoteFs`, queue and workers as the app; a `get` measures the same throughput as the GUI on the same file.
- `xsb reachable <site>` is what CI uses to smoke the Docker server.

### E15 — Testing and quality

#### E15-S1 — In-process SSH test server
A `russh::server` fixture that accepts a test key, spawns a local `xs
--server` on exec, an in-process SFTP server (`russh-sftp` server side) and a
PTY-less shell stub, so engine tests need no Docker.
**AC**
- `cargo test --workspace` runs all E2–E7 engine tests against it in < 60 s.

#### E15-S2 — Docker backend matrix (nightly)
`openssh-server` with `xs` installed, netem profiles (LAN, 50 ms, 200 ms) for
the throughput gates in E4-S1 and E6-S4.
**AC**
- Results are appended to `docs/bench.md` with date and host.

#### E15-S3 — Visual regression
`test-support` screenshots of the kitchen-sink screens on macOS and Linux
compared against committed baselines with a tolerance.
**AC**
- A one-pixel row-height change in the file table fails the job.

#### E15-S4 — Performance gates
Frame-time budget with `gpui-fps` for: 50k-row pane, 10k-row queue with 3
active, terminal at 50 MB/s output.
**AC**
- Gates are numbers in CI, not prose.

### E16 — Packaging and release

#### E16-S1 — Bundling `xs`
Ship the matching `xs` binary beside the app for local use and for a future
bootstrap: macOS `Contents/MacOS/xs`, Linux next to the binary, Windows
next to the `.exe`. The app version string names the embedded xsync version.
**AC**
- `xsaber --version` prints both versions; the suite tag records them.

#### E16-S2 — macOS bundle
`cargo-bundle` or `cargo-packager` producing `xsaber.app` with icon, fonts and
`Info.plist`; ad-hoc signed in CI; Developer ID signing and notarisation
documented as a manual release step until the suite decides on signing.
**AC**
- The bundle opens on a clean macOS VM after right-click → Open.

#### E16-S3 — Linux packages
AppImage and `.deb` with desktop file and icon; Wayland and X11 tested.
**AC**
- Runs on Ubuntu 24.04 and Fedora 41 runners.

#### E16-S4 — Windows
MSI via `cargo-packager`; keychain via Windows Credential Manager; Pageant
agent; console-less launch.
**AC**
- CI Windows job leaves `continue-on-error`; the app connects to the Docker server from a Windows runner.

**Blocker:** `xsync-core` currently imports Unix-only `rustix::fs`, which makes
the allowed E0 Windows CI job fail. The provider must gain Windows support
before this story can close its Windows connection acceptance criterion.

### E17 — Suite integration

#### E17-S1 — Submodule registration
Register the published `s4njee/xsaber` `xcalibur` commit in the superproject:
add the `.gitmodules` entry with `branch = xcalibur`, pin the SHA, and update
`../README.md`, `../AGENTS.md`, and `../ROADMAP.md` progress. E0-S1 owns
creating and publishing the repository; this story owns only suite registration
and the root pin.
**AC**
- `git submodule status` at the suite root shows six clean pins.

#### E17-S2 — xsync prerequisite stories (provider side)
File in `../xsync/backlog.md`, cited from `integration.md`, prioritised:
1. `Cancel` sent from `xsync-client` for in-flight requests (server already handles type 18).
2. `OWNER_NAMES` granted so `Attrs.names` is populated (perms/owner columns).
3. Remote-binary override and non-`ssh` transport in `connect_ssh`, or a documented `from_stream` recipe (this plan uses `from_stream`; the doc is the deliverable).
4. Server-side recursive delete and `mkdir -p` verbs (client loops until then).
5. Notify/watch (types 110–115) for live refresh; xsaber polls per `dir_cache_ms` until then.
6. `read_dir_path` overflow: a paged variant so large directories do not fall back to the handle path.
**AC**
- Each item has an id in xsync's backlog and a consumer story here that flips from workaround to native when it lands. For the two capabilities whose baseline stories are already complete but which have no distinct open provider ID, `integration.md` keeps the exact missing-ID follow-up caveat; E17-S2 remains open until xsync assigns and scopes both follow-ups.

**Blocker:** xsync has not assigned or scoped distinct follow-up IDs for
server-side recursive delete/parent-creating `mkdir`, or for a paged
`read_dir_path` overflow API. Until it does, xsaber retains the client-side
walk/one-level-mkdir and directory-handle fallbacks documented in
`integration.md`.

#### E17-S3 — Shared-crate decision
xsaber is the third GUI, the roadmap's trigger for considering a shared crate
(progress/event DTOs, `RemoteFs`/`ShareBackend`, hostkeys, secrets). Write
`../docs/gui-conventions.md` first (suite rule), then decide.
**AC**
- A dated decision in `../ROADMAP.md` §5 either opens the extraction (with the crate name and owner) or records why not.

#### E17-S4 — Docs that describe the port
`integration.md` names every file ported from `../excalibur/crates/engine`
with its excalibur commit, so fixes can be cross-applied until E17-S3 lands.
**AC**
- A script (`scripts/port-diff.sh`) diffs each ported file against its source and lists drift.

---

## Milestones

| Milestone | Epics / stories | Demo |
|---|---|---|
| **M0 — Chrome** | E0, E1 | The kitchen-sink example opens on macOS, Linux and Windows CI runners showing all five mock screens with the real theme, fonts and icons, and the empty main window has working menus and window controls. |
| **M1 — Connect and browse** | E2, E3, E4, E5, E8, E9, E11-S1…S3 | `⌘K`, type `deploy@node-04`, watch the handshake log, trust the key, and browse `/var/www` over xsync in the right pane and `~/projects` in the left, with sort, selection, rename and delete. |
| **M2 — Transfer** | E6, E10-S1…S3 | Drag four files to the remote pane; the queue drains at three workers with live progress, one row fails and retries, `.env.production` is skipped, history records the batch, and the status bar shows throughput. |
| **M3 — Terminal** | E7 | Run `df -h /var/www` in the drawer while a 1 GiB upload runs on the same session; "Open in terminal" on a remote folder `cd`s the shell; `⌃\`` collapses and restores it. |
| **M4 — Manage** | E12, E13, E11-S4, E10-S4 | Save the site with a key file, reconnect from the site manager, change concurrency in Preferences and watch the queue adapt without reconnecting. |
| **M5 — Ship** | E14, E15, E16, E17 | `xsb get` from CI against the Docker server; signed-enough macOS and Linux bundles; xsaber pinned as the suite's sixth submodule with green CI on macOS and Linux and a reported Windows job. |

## Definition of done (v1)

- [ ] All M0–M5 stories marked Done with dated evidence in this file's Progress table.
- [ ] `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `cargo deny check` green on macOS and Linux; Windows job reported.
- [ ] Connect over xsync and over SFTP fallback to the same host with one host-key prompt.
- [ ] Terminal drawer survives tab switches, drawer collapse and a concurrent transfer.
- [ ] No secret in argv, logs, events, persisted queue, sites file or `Debug` output (test-enforced).
- [ ] `docs/design-gaps.md` lists every deviation from the handoff.
- [ ] `../.gitmodules` pins xsaber; `../README.md` and `../AGENTS.md` describe it.

## Risks and open questions

- **GPUI churn.** `gpui-kit` renamed and re-architected on 2026-09-03 and depends on `gpui-pre`, a third-party weekly snapshot of Zed's GPUI. Pin exact versions in `Cargo.lock`, upgrade deliberately, and keep the kitchen sink as the canary.
- **Terminal from scratch.** No GPUI-family project ships a terminal. E7 is the largest UI story; Zed's `crates/terminal` is the reference and it maintains an `alacritty_terminal` fork. Budget a spike (E7-S1 + a throwaway renderer) before M1 ends.
- **russh-sftp read throughput** (un-pipelined reads; issue #549 on large reads). E4-S1's gate exists to catch this early; the escape hatch is our own ranged-read loop over `RawSftpSession`.
- **Shipping `xs`.** The suite's only pattern is a Tauri sidecar. E16-S1 is new ground; the alternative is documenting "install xsync on the host" and shipping nothing.
- **Windows.** Every provider in the suite is unverified there. v1 keeps Windows building, not shipping.
- **Design-system ownership.** The handoff originated its palette because no brand system existed. If one appears, E1-S1's JSON is the single file to change.
- **Sync tab.** The design reserves a tab; xsync's planner could back it. Out of v1 scope; revisit after M5 with `run_client_push` or a v3-native plan/apply once xsync exposes one.
