# Handoff: xsaber — FTP/SFTP client UI

## Overview
xsaber is a desktop FTP/SFTP transfer client (comparable in category to FileZilla, Cyberduck, WinSCP). This bundle covers five screens of a dark, high-density, cross-platform desktop shell built on a **modernized dual-pane model**: local files on the left, remote files on the right, with an **attached terminal drawer docked at the bottom** of the dual pane. The drawer is the distinguishing element — it hosts a live shell session on the connected host alongside the transfer queue and message log, so the user never leaves the app to run a deploy command.

Audience: mixed/general purpose, leaning pro — sysadmins, web developers deploying builds. Density was an explicit requirement ("very dense, pro tool"): 25px file rows, 26px table headers, no decorative whitespace.

## About the Design Files
The files in this bundle are **design references created in HTML** — prototypes showing intended look and behavior, not production code to copy directly. `xsaber UI.dc.html` is a single streaming HTML design file with all styling inline and its sample data supplied by a small JavaScript class at the bottom of the file. It is not a component library and has no build step.

The task is to **recreate these designs in the target codebase's existing environment** (Electron + React, Tauri, Qt, SwiftUI, WinUI, etc.) using its established patterns, component library, and theming layer. Where this document gives a hex value or a pixel measurement, treat it as the intended visual result, not as a mandate to hand-write inline styles — map it onto the codebase's tokens.

If no environment exists yet: this is a desktop app with a native-feeling window chrome, a persistent shell session (PTY), and file-system access on both ends. Electron or Tauri with React is the pragmatic choice; the design assumes a custom-drawn title bar rather than the OS one.

## Fidelity
**High-fidelity.** Final colors, typography, spacing, and copy. Recreate the UI closely, substituting the codebase's own primitives (buttons, tables, tabs, modals) where they can carry the same visual result. Two caveats:

- These are **static mockups**. No interaction was built, so behavior below is specified in prose, not demonstrated.
- Icons are **placeholders**: simple hand-drawn inline SVG strokes (refresh, upload, download, sync, filter, terminal, search, lock, spinner). Replace all of them with the codebase's real icon set. No production icon library was available.
- There is **no logo**. The wordmark uses a placeholder blade glyph (a small skewed parallelogram) next to the text "xsaber". Replace with the real mark when it exists.

## Screens / Views

All five screens live in one HTML file as a canvas board, each wrapped in a labelled container with a stable id (`1a`–`1e`).

---

### 1a — Main transfer window
**Purpose:** The app's home. Browse local and remote trees side by side, select files, queue transfers, and run shell commands on the connected host without leaving the window.

**Frame:** 1420 × 920, background `#0c0d10`, 1px border `#24282f`, radius 10px, drop shadow `0 24px 60px rgba(0,0,0,.6)`. Vertical stack of six fixed-height bands plus one flexible band:

| Band | Height | Background |
|---|---|---|
| Title bar | 40px | `#14161b` |
| Session tabs | 34px | `#101216` |
| Toolbar | 46px | `#14161b` |
| Dual pane | 480px (flexible in production) | `#0c0d10` |
| Terminal drawer | 220px (user-resizable) | `#0a0b0e` |
| Status bar | 26px | `#14161b` |

Every band is separated by a 1px `#24282f` rule.

#### Title bar (40px)
Left to right, with `padding: 0 12px 0 14px` and an 18px gap between groups:
- **Wordmark**: 16×16 placeholder blade SVG (`#ff6b3d` body, `#ff8b5f` highlight facet) + text "xsaber" at Barlow 700 15px, `letter-spacing: -0.2px`, color `#e6e8ec`, with the leading "x" in `#ff6b3d`.
- **Menu bar**: File · Edit · View · Transfer · Server · Bookmarks · Help. Barlow 500 12px, `#9aa1ad`, 16px gap. In-window menu bar on all platforms (do not defer to the macOS system menu in the mock; on real macOS builds mirror these into the system menu).
- **Spacer** (flex: 1).
- **Protocol badge**: "SFTP · ED25519", Barlow 500 10px, color `#3ecfb2`, background `rgba(62,207,178,.10)`, 1px border `rgba(62,207,178,.22)`, radius 4px, `padding: 4px 8px`, preceded by a 5px `#3ecfb2` dot. The teal is reserved system-wide for "secure / verified / succeeded".
- **Window controls**: `–` `□` `×`, Barlow 400 13px, `#6b7280`, 14px gap. Placeholder chrome — use platform-appropriate controls (traffic lights on macOS, right-side controls on Windows/Linux).

#### Session tabs (34px)
Full-bleed to the left edge (`padding: 0 8px 0 0` — the first tab must sit flush with the window edge; this was an explicit correction). Each tab: `padding: 0 14px`, a 5px status dot, then the label at Barlow 11.5px.

- Active tab: background `#1a1d24`, `border-top: 2px solid #ff6b3d`, label weight 600 `#e6e8ec`.
- Inactive: transparent, weight 500 `#8b93a1`.
- Status dot colors: `#3ecfb2` connected, `#4b525e` idle/disconnected, `#ffb020` connected-but-degraded or unencrypted FTP.
- Content: "node-04 · prod" (active, teal), "staging-eu" (grey), "cdn-assets" (amber), then a `+` at Barlow 400 14px `#6b7280`.

#### Toolbar (46px)
`padding: 0 12px`, 8px gap.

- **Quick connect strip**: one 28px-tall segmented control, background `#0c0d10`, 1px border `#2e333c`, radius 5px, `overflow: hidden`. Alternating label/value cells divided by 1px `#24282f` verticals. Labels: Barlow 500 9.5px `#6b7280`, `padding: 0 8px`. Values: Barlow 400 11.5px `#e6e8ec`, `padding: 0 10px`. Fields and widths: HOST (196px, "node-04.xsaber.dev"), USER (78px, "deploy"), PORT (44px, "22"). In production these are text inputs; the mock draws them as filled values.
- **Connect button** (28px): **outlined, not solid** — background `rgba(255,107,61,.08)`, 1px border `#ff6b3d`, text `#ff6b3d` Barlow 600 11.5px, radius 5px, `padding: 0 14px`. This treatment applies to every Connect button in the app (an explicit design decision: the primary action reads as an outline with a mostly-dark fill).
- **Divider**: 1px × 22px `#2e333c`, `margin: 0 4px`.
- **Icon buttons**: 28×28, radius 5px, 2px gap, 14px placeholder SVG at 1.3 stroke `#c7ccd6`. In order: Refresh (has resting background `#1a1d24` + border `#2e333c`, i.e. the default enabled style), Upload, Download, Synchronized browsing, then a divider, then Filter (funnel), then **Terminal** — the one toggled-on button: background `rgba(255,107,61,.12)`, border `rgba(255,107,61,.3)`, stroke `#ff8b5f`.
- **Spacer**, then two right-aligned 26px chips: "Filter files" (search glyph + Barlow 400 11px `#6b7280`, background `#0c0d10`, border `#2e333c`) and "4 QUEUED" (Barlow 500 10.5px `#ffb020`, same shell). Amber is the queued/pending color.

#### Dual pane (480px)
Two equal flex columns (`flex: 1; min-width: 0`) separated by a 1px `#24282f` border. Both panes are structurally identical — build one component, instantiate twice.

**Pane header (36px)**, background `#101216`, bottom border `#1e222a`, `padding: 0 10px`, 10px gap:
- Side chip: "LOCAL" / "REMOTE", Barlow 600 9.5px, `letter-spacing: .08em`, `#8b93a1`, background `#1a1d24`, radius 3px, `padding: 4px 6px`.
- Lock glyph (11px, `#3ecfb2`) — remote pane only, shown when the connection is encrypted.
- Path: Barlow 400 11.5px `#c7ccd6`, flexes, truncates with ellipsis. Editable breadcrumb/path field in production.
- Meta, right aligned: Barlow 400 10.5px `#6b7280`. Local: "14 items · 4 selected · 1.8 MB". Remote: "13 items · 3 writing".

**Column header (26px)**, background `#14161b`, bottom border `#24282f`, `padding: 0 10px`. Barlow 600 9.5px, `letter-spacing: .06em`, `#6b7280`. Labels: NAME, SIZE (right aligned), TYPE, MODIFIED, PERMS.

**File rows (25px each)** — grid `minmax(0,1fr) 76px 100px 128px 96px`, `padding: 0 10px`, bottom border 1px `#14161b`. Both panes use the identical grid so the two tables read as one ruler.
- **Name cell**: 8px gap, an extension badge then the filename. Badge: Barlow 600 8.5px, `padding: 3px 4px`, radius 2px, `min-width: 24px`, centered. Filename: Barlow 400 12px, truncating.
- **Size**: Barlow 400 11px `#8b93a1`, right aligned. **Type**: Barlow 400 11px `#6b7280`, `padding-left: 12px`. **Modified**: Barlow 400 11px `#6b7280`. **Perms**: Barlow 400 10.5px `#4b525e` (deliberately the dimmest column — present for pros, not competing for attention).
- **Row states**: selected → background `rgba(255,107,61,0.09)`; queued-for-write (remote side) → `rgba(255,176,32,0.05)`; default transparent. Filename color `#e6e8ec`, or `#6b7280` for de-emphasized rows (`..`, superseded builds, lock files, dotfiles).

**Extension badge palette** — keyed by file class, background / foreground:
| Class | Background | Foreground |
|---|---|---|
| DIR | `rgba(122,162,255,.14)` | `#7aa2ff` |
| JS | `rgba(255,199,64,.13)` | `#ffc740` |
| CSS | `rgba(98,182,255,.13)` | `#62b6ff` |
| HTM | `rgba(255,107,61,.15)` | `#ff8b5f` |
| PHP | `rgba(155,123,255,.15)` | `#b39bff` |
| SVG | `rgba(62,207,178,.13)` | `#3ecfb2` |
| ENV (secrets) | `rgba(255,77,79,.13)` | `#ff7a7c` |
| MD / XML / TXT / ICO / CFG (neutral) | `rgba(255,255,255,.06)` | `#9aa1ad` (`#8b93a1` for CFG) |

Badges replace file-type icons entirely — a density decision. Keep them if you like the look; if you swap in real icons, keep the 24px cell width so the name column stays aligned.

**Sample content.** Local `~/projects/atlas-web/dist`: `..`, `assets` (DIR), `index.html`, `app.b41f9c.js`, `app.b41f9c.css`, `vendor.9a2e14.js` (these four selected), `logo-mark.svg`, `favicon.ico`, `manifest.webmanifest`, `robots.txt`, `sitemap.xml`, `CHANGELOG.md`, `.env.production` (dimmed, perms `-rw-------`), `build-report.json`. Remote `/var/www/atlas/releases/20260907`: `..`, `assets`, `logs`, `index.html`, `app.b41f9c.js`, `app.7fd11a.js` (dimmed — previous build), `app.b41f9c.css` (0 B, mid-write), `vendor.9a2e14.js`, `favicon.ico`, `robots.txt`, `.htaccess`, `healthz.php`, `release.lock` (dimmed). The scenario is deliberate: a build being uploaded into a timestamped release directory, mid-transfer.

#### Terminal drawer (220px)
The signature element. Docked to the bottom of the dual pane, above the status bar, background `#0a0b0e`. Should be drag-resizable and collapsible.

**Tab bar (30px)**, background `#101216`, bottom border `#24282f`, `padding: 0 6px`:
- Tabs: **Terminal** (active), **Transfer queue** with a count badge, **Message log**, **Sync**. Active tab: `border-bottom: 2px solid #ff6b3d`, Barlow 600 11px `#e6e8ec`, plus a 12px terminal glyph in `#ff8b5f`. Inactive: Barlow 500 11px `#8b93a1`. `padding: 0 12px`.
- Count badge: "4", Barlow 600 9.5px `#ffb020` on `rgba(255,176,32,.14)`, radius 3px, `padding: 3px 5px`.
- Right side: "deploy@node-04:~ · bash" at Barlow 400 10.5px `#4b525e`, then the shortcut hint `⌃\`` and a collapse chevron `⌄` in `#6b7280`.

**Terminal body**, `padding: 10px 12px`. **This is the only place in the app that uses a monospace font** — an explicit instruction. Every other surface is Barlow.
- Font: IBM Plex Mono 400 11.5px / 1.55.
- Command lines: prompt `deploy@node-04 ~ $` in `#ff6b3d`, 8px gap, command in `#e6e8ec`.
- Output lines: `#7f8794`, `padding-left: 2px`.
- Blocks separated by `margin-bottom: 6px`.
- Final line: prompt + a 7×14px `#ff6b3d` block cursor, animated with the `pulse` keyframe (opacity .35 → 1 → .35, 1.1s ease-in-out infinite).
- Sample session: `df -h /var/www` (filesystem table), `ln -sfn releases/20260907 current && systemctl reload nginx` (nginx config test output), `tail -n 2 logs/access.log` (two access-log lines).

#### Status bar (26px)
`padding: 0 12px`, 14px gap, Barlow 400 10.5px `#6b7280`. Left: "Connected" in `#3ecfb2`, then "sftp://deploy@node-04.xsaber.dev:22". Right: "4 queued · 1 active", "↑ 8.4 MB/s" (`#c7ccd6`), "↓ 0 B/s", "latency 24 ms".

---

### 1b — Site manager
**Purpose:** Manage saved connections, grouped by environment; edit one site's credentials and default directories; connect from the dialog.

**Frame:** 900 × 558 total. Background `#0c0d10`, 1px border `#2e333c`, radius 10px, same shadow as 1a. Header 40px + body 480px + footer 46px.

- **Header (40px)**: "Site manager", Barlow 600 13px `#e6e8ec`, `padding: 0 14px`, background `#14161b`, bottom border `#24282f`; `×` at Barlow 400 13px `#6b7280` on the right.
- **Body**: two columns, 480px tall.

**Left column (274px)**, background `#0a0b0e`, right border `#24282f`:
- Header strip (28px): "SITES", Barlow 600 9.5px `letter-spacing: .06em` `#6b7280`, bottom border `#1e222a`.
- Tree rows (27px each), 8px gap, `padding-left: 10px` for groups and **26px for children** (that indent is the only hierarchy cue). Each row: a tag badge (Barlow 600 8.5px, radius 2px, `min-width: 26px`, centered) then the name at Barlow 400 12px.
- Tag badge palette: group `rgba(255,255,255,.05)` / `#8b93a1` (the badge content is a `▾` disclosure glyph); SFTP `rgba(62,207,178,.12)` / `#3ecfb2`; FTPS `rgba(122,162,255,.13)` / `#7aa2ff`; FTP `rgba(255,176,32,.13)` / `#ffb020` (amber = unencrypted, a soft warning); S3 `rgba(155,123,255,.14)` / `#b39bff`.
- Selected row: background `rgba(255,107,61,0.10)`, name `#e6e8ec`; otherwise `#c7ccd6`.
- Content: **Atlas production** → node-04 · prod (selected), node-05 · prod, cdn-assets; **Staging** → staging-eu, staging-us; **Legacy clients** → hollis-media.net (FTPS), kestrel-print.co (FTP), archive.dev-1998 (FTP).
- Footer strip (34px), top border `#1e222a`: "New site" and "Folder" buttons (Barlow 500 11px `#c7ccd6`, background `#1a1d24`, border `#2e333c`, radius 4px, `padding: 5px 9px`), spacer, "Duplicate" as a borderless `#8b93a1` action.

**Right column** (flex, `min-width: 0`):
- Tab strip (32px), background `#101216`, bottom border `#24282f`: General (active — `border-bottom: 2px solid #ff6b3d`, Barlow 600 11px `#e6e8ec`), Advanced, Transfer, Charset (Barlow 500 11px `#8b93a1`). `padding: 0 12px` each.
- Form, `padding: 16px 18px`, 11px row gap. Each row is a grid `104px minmax(0,1fr)` with a 14px gap: label right-aligned at Barlow 500 11px `#8b93a1`, then a 28px field — background `#0a0b0e`, 1px border `#2e333c`, radius 5px, `padding: 0 10px`, value at Barlow 400 11.5px, with a right-aligned hint at Barlow 400 10px `#4b525e`.
- Rows (label / value / hint): Protocol / "SFTP — SSH File Transfer" / `▾`; Host / "node-04.xsaber.dev"; Port / "22" / "default"; Logon type / "Key file" / `▾`; User / "deploy"; Key file / "~/.ssh/id_ed25519" / "Browse…"; Remote dir / "/var/www/atlas/releases"; Local dir / "~/projects/atlas-web/dist"; Comment / "Blue-green deploy target — do not delete releases/". Value color `#e6e8ec`, dropping to `#c7ccd6` for the two directory rows and `#8b93a1` for the comment.
- Below the form, in the value column: three status chips at Barlow 500 10.5px, radius 4px, `padding: 5px 8px` — "HOST KEY VERIFIED" (`#3ecfb2` on `rgba(62,207,178,.10)`, border `rgba(62,207,178,.24)`), "SHA256:9f2a…c41d" and "last used 2h ago" (both `#8b93a1` on `#14161b`, border `#2e333c`).
- **Footer (46px)**, background `#101216`, top border `#24282f`, `padding: 0 16px`, 8px gap: "Delete" (borderless `#8b93a1`), spacer, "Cancel" and "Save" (Barlow 11.5px `#c7ccd6`, background `#1a1d24`, border `#2e333c`, radius 5px, `padding: 7px 13px`), then **Connect** in the outlined orange treatment (`rgba(255,107,61,.08)` fill, `#ff6b3d` border and text, `padding: 6px 14px`).

---

### 1c — Quick connect, and connecting
Two 400px-wide cards shown side by side (20px gap). Both: background `#0c0d10`, border `#2e333c`, radius 9px, shadow `0 20px 46px rgba(0,0,0,.6)`.

**Card 1 — Quick connect popover.** Invoked by `⌘K`; the header (34px, `#14161b`) reads "QUICK CONNECT" at Barlow 600 9.5px `letter-spacing: .07em` `#8b93a1`, with the shortcut hint right-aligned in `#4b525e`.
- Body `padding: 14px`, 9px gap.
- **Focused URL field** (32px): background `#0a0b0e`, **1px border `#ff6b3d`** (focus ring color), radius 5px. Contents: scheme prefix "sftp://" at Barlow 600 9.5px `#6b7280`, the value "deploy@node-04.xsaber.dev" at Barlow 400 12px `#e6e8ec`, then a 6×14px `#ff6b3d` caret using the same `pulse` animation.
- **Second row**: grid `1fr 84px`, 8px gap, two 30px fields (background `#0a0b0e`, border `#2e333c`, radius 5px) — "Key: ~/.ssh/id_ed25519" (`#6b7280`, placeholder-toned) and "22" (`#c7ccd6`).
- **Recents list**: section label "RECENT" (Barlow 600 9.5px `#4b525e`, `padding: 6px 2px`), then 28px rows with a 5px status dot, host at Barlow 400 11.5px `#c7ccd6`, and a right-aligned relative time at Barlow 400 10.5px `#4b525e`. First row is highlighted `rgba(255,107,61,0.07)` (keyboard cursor). Content: deploy@node-04.xsaber.dev / 2h ago / teal dot; deploy@staging-eu.xsaber.dev / yesterday / grey; ftp@hollis-media.net / 4 Sep / amber; ci@cdn-assets.xsaber.dev / 28 Aug / grey.
- **Actions**: "Connect" (outlined orange, `flex: 1`, centered, `padding: 8px`) and "Save site" (`#1a1d24` / `#2e333c` / `#c7ccd6`, `padding: 9px 12px`).

**Card 2 — Connecting.** Header (34px): a 12px spinner SVG stroked `#ff6b3d`, animated with the `spin` keyframe (360° / 900ms linear infinite), then "CONNECTING" at Barlow 600 9.5px `#e6e8ec`, spacer, "Cancel" at Barlow 400 10.5px `#6b7280`.
- **Progress bar**: 2px track `#1a1d24` with a 64%-wide `#ff6b3d` fill, flush under the header.
- **Handshake log**, `padding: 13px`, Barlow 400 11.5px / 1.7, three columns per line: elapsed time (`#4b525e`, `min-width: 52px`), a status mark (`min-width: 12px`), message (`#9aa1ad`). Marks: `✓` in `#3ecfb2` for done, `●` in `#ff6b3d` for in-progress, blank for not-yet-started. Lines: 0.004s resolved host → 10.4.2.19; 0.021s TCP established on port 22; 0.088s "Server: OpenSSH_9.6p1 · SSH-2.0"; 0.140s key exchange curve25519-sha256; 0.212s "Verifying host key fingerprint…" (in progress); then "Authenticating deploy with id_ed25519" (pending).
- **Footer prompt**, `padding: 11px 13px`, top border `#1e222a`, background `#0a0b0e`: a "NEW HOST KEY" chip (Barlow 600 9.5px `#ffb020` on `rgba(255,176,32,.13)`, radius 3px) then "SHA256:9f2a…c41d — trust and continue?" at Barlow 400 10.5px `#7f8794`. In production this needs Trust / Trust once / Reject actions.

---

### 1d — Drawer expanded: transfer queue & history
**Purpose:** The Transfer queue tab of the same bottom drawer from 1a, dragged tall. Shown in the mock as a standalone 1420-wide card (background `#0a0b0e`, border `#24282f`, radius 10px) — in the app it is the drawer region, not a separate window.

- **Tab bar (30px)**: identical to 1a's, but "Transfer queue" is the active tab (orange underline, count badge "4") and "Terminal" is inactive. Right side: "↑ 8.4 MB/s" (`#c7ccd6`) then "· 3 workers · retry ×2" at Barlow 500 10.5px `#6b7280`, plus the collapse chevron.
- **Queue table header (26px)**, background `#14161b`, grid `minmax(0,1fr) 84px 220px 84px 72px 96px`, `padding: 0 12px`, Barlow 600 9.5px `letter-spacing: .06em` `#6b7280`: FILE, SIZE (right), PROGRESS (`padding-left: 16px`), SPEED (right), ETA (right), STATE (`padding-left: 16px`).
- **Queue rows (32px)**, same grid, bottom border `#14161b`:
  - **File cell**: 9px gap — a direction glyph (`↑` in `#ff8b5f` for upload, `↓` in `#62b6ff` for download, Barlow 600 11px, 9px wide), the filename at Barlow 400 12px `#e6e8ec`, then the route at Barlow 400 10.5px `#4b525e` (e.g. "dist → releases/20260907"), both truncating.
  - **Progress cell**: a flexible 4px track `#1a1d24` radius 2px with a fill of the row's percentage, then the percentage text at Barlow 500 10px `#7f8794` in a 32px right-aligned column. Fill color: `#ff6b3d` while transferring, `#ff4d4f` on failure, `#2e333c` when queued or already complete.
  - **State badge**: Barlow 600 9px, `letter-spacing: .05em`, radius 3px, `padding: 4px 6px`. TRANSFERRING → `rgba(255,107,61,.14)` / `#ff8b5f`; QUEUED → `rgba(255,255,255,.06)` / `#8b93a1`; COMPLETE → `rgba(62,207,178,.12)` / `#3ecfb2`; FAILED → `rgba(255,77,79,.13)` / `#ff7a7c`; SKIPPED → `rgba(255,176,32,.13)` / `#ffb020`.
  - Active row background: `rgba(255,107,61,0.05)`; others transparent.
  - Rows: vendor.9a2e14.js 1.2 MB 64% 8.4 MB/s 0:04 TRANSFERRING · app.b41f9c.css 88.4 KB QUEUED · index.html 4.2 KB QUEUED · sitemap.xml 9.8 KB QUEUED · assets/hero-2400.webp 842 KB 100% COMPLETE · ↓ logs/access.log 24.6 MB 100% COMPLETE · .env.production 740 B SKIPPED · app.b41f9c.js 486 KB 38% FAILED.
- **History section header (26px)**, background `#101216`, borders top and bottom `#24282f`: "HISTORY · TODAY" (Barlow 600 9.5px `letter-spacing: .06em` `#6b7280`), right-aligned summary "18 transfers · 412.6 MB · 1 failed" in `#4b525e`.
- **History rows (27px)**, grid `64px minmax(0,1fr) 84px 96px 120px`, `padding: 0 12px`, bottom border `#101216`, Barlow 400 11px: clock time `#4b525e`; direction glyph inline with the path, `#9aa1ad`, truncating; size `#6b7280` right; duration `#4b525e` right; state badge (same palette). Rows: 14:12:06 hero-2400.webp 842 KB 0.9 s COMPLETE · 14:11:52 ↓ logs/access.log 24.6 MB 3.6 s COMPLETE · 14:11:20 robots.txt 128 B 0.1 s COMPLETE · 14:10:44 app.7fd11a.js 478 KB FAILED · 14:09:58 .DS_Store 6 KB SKIPPED.

---

### 1e — Preferences → Transfers
**Purpose:** App settings. The Transfers section is shown; other sections are listed but not designed.

**Frame:** 940 × 652 — header 40px + body 566px + footer 46px. Same card treatment as 1b.

- **Header (40px)**: "Preferences" (Barlow 600 13px `#e6e8ec`), spacer, a 26px "Search settings" chip (search glyph + Barlow 400 11px `#6b7280`, background `#0a0b0e`, border `#2e333c`, radius 5px).
- **Left nav (194px)**, background `#0a0b0e`, right border `#24282f`, `padding: 8px 0`. Rows 30px, `padding: 0 12px`, Barlow 500 12px. Items: Connection, **Transfers** (active), Interface, Terminal, Security, File editing, Updates. Active row: background `rgba(255,107,61,0.09)`, `border-left: 2px solid #ff6b3d`, text `#e6e8ec`; inactive text `#9aa1ad` with a transparent 2px left border (keeps text aligned).
- **Settings pane** (flex, `padding: 16px 20px`, 14px gap between groups). Each group: a title at Barlow 600 9.5px `letter-spacing: .07em` `#6b7280` with a 9px bottom pad and a 1px `#1e222a` rule under it, then rows.
- **Setting row**: `padding: 9px 0`, bottom border `#101216`, 16px gap. Left side is a label (Barlow 500 12px `#e6e8ec`) over help text (Barlow 400 11px / 1.4 `#6b7280`). Right side is one control:
  - **Toggle**: 34 × 18px pill, radius 9px, `padding: 0 2px`, with a 14px round knob. On: track `#ff6b3d`, knob `#1a0d07`, knob right. Off: track `#24282f`, knob `#6b7280`, knob left.
  - **Value field**: 27px tall, background `#0a0b0e`, border `#2e333c`, radius 5px, `padding: 0 10px`, value at Barlow 400 11.5px `#c7ccd6`, right-aligned suffix (`▾`, a unit, or "edit") in `#4b525e`. `min-width` varies per row (96–190px) — size to content.
- **Groups and rows:**
  - **CONCURRENCY** — "Simultaneous transfers" / "Per session. Higher values saturate slow links." / value `3 ▾`; "Speed limit — upload" / "Applies to all sessions" / value `unlimited KB/s`; "Split large files into chunks" / "Files above 64 MB are transferred in parallel parts" / toggle **on**.
  - **CONFLICTS & INTEGRITY** — "When the file exists" / "Default action for the overwrite dialog" / value `Overwrite if newer ▾`; "Verify checksums after upload" / "Compares SHA-256 on both ends; slows large batches" / **on**; "Resume interrupted transfers" / "Restarts from the last confirmed byte" / **on**; "Preserve timestamps" / "Requires server support (MFMT / SFTP setstat)" / **off**.
  - **FILTERS** — "Ignore patterns" / "Skipped in both directions" / value `.DS_Store, .git/, *.log` with an "edit" affordance; "Warn before uploading dotfiles" / "Catches .env and credential files" / **on**.
- **Footer (46px)**: "Restore defaults" (borderless `#8b93a1`), spacer, "Cancel" (`#1a1d24` / `#2e333c` / `#c7ccd6`), "Apply" — currently the one **solid** orange button in the app (`#ff6b3d` fill, `#1a0d07` text, radius 5px, `padding: 7px 15px`). If you prefer full consistency, switch it to the outlined Connect treatment; it was deliberately left solid pending a decision.

---

## Interactions & Behavior
Not built in the mocks. Intended behavior:

- **Panes**: double-click a folder to descend; `..` ascends. Multi-select with click, shift-click (range), cmd/ctrl-click (toggle). Drag a selection to the opposite pane to queue a transfer; the toolbar's up/down arrows do the same for the active pane's selection. Column headers sort. Right-click for a context menu (download/upload, rename, delete, permissions, copy URL, open in terminal — the last should `cd` the drawer's shell to that directory). Synchronized browsing keeps both paths in step when the trees mirror each other.
- **Connect flow**: quick connect or site manager → the connecting card's handshake log fills line by line, marks flipping from blank to `●` to `✓`. An unrecognized host key blocks on the amber prompt until the user trusts or rejects. On success the session tab's dot turns teal and both panes populate.
- **Transfers**: the queue drains at N workers (the Concurrency setting). A transferring row animates its progress fill and updates speed/ETA roughly every 250ms — keep updates coalesced; the table is dense enough that per-row re-render will show. Failures stay in the queue with a FAILED badge and a retry affordance. The status bar's aggregate throughput and the "4 QUEUED" toolbar chip both track queue state.
- **Drawer**: `⌃\`` toggles collapsed/expanded and focuses the terminal. Drag the top edge to resize (suggested range 120px–60% of window height); double-click the tab bar to collapse. The shell is a real PTY over the existing SSH connection and should survive tab switches within the drawer. Switching the Transfer queue tab must not kill the terminal session.
- **Hover states** (unspecified in the mock — recommended): file rows lift to `rgba(255,255,255,.03)`; icon buttons take background `#1a1d24` with border `#2e333c`; tabs brighten label to `#c7ccd6`; the outlined Connect button raises its fill to `rgba(255,107,61,.16)`. Focus rings: 1px `#ff6b3d` (matching the quick-connect field).
- **Responsive**: desktop-only, fixed chrome. The dual pane splits 50/50 by default with a draggable divider; the file table's flexible NAME column absorbs width change while the five fixed columns hold. Below roughly 900px window width, the mock's density stops working — consider collapsing to a single pane with a target selector.

## State Management
- **Sessions**: array of connections, each with `{host, port, protocol, user, auth, status: connected|connecting|idle|error, localPath, remotePath, hostKeyVerified}`. Active session index drives the tabs.
- **Directory listings**: per pane, `{path, entries[], loading, error, selection: Set<id>, sortKey, sortDir}`. Cache per path with explicit refresh.
- **Transfer queue**: ordered array of `{id, direction, localPath, remotePath, bytesTotal, bytesDone, speed, eta, state}` plus derived aggregates (count, active count, total throughput) consumed by the status bar and toolbar chip. History is the completed/failed tail, persisted per day.
- **Terminal**: PTY handle per session, scrollback buffer, current working directory (worth syncing with the remote pane's path).
- **Preferences**: a single settings object, persisted to disk, read by the transfer engine (concurrency, conflict policy, checksum verification, ignore patterns).
- **Data fetching**: all of it is protocol I/O (SFTP/FTP listings, transfers) plus local FS reads — run it off the UI thread and stream progress events.

## Design Tokens

**Colors — surfaces**
| Token | Hex | Use |
|---|---|---|
| canvas | `#08090b` | Board background behind the mock windows |
| bg | `#0c0d10` | Window body, file panes, dialogs |
| bg-sunken | `#0a0b0e` | Terminal drawer, left rails, input fields |
| bg-strip | `#101216` | Tab strips, pane headers, footers |
| surface | `#14161b` | Title bar, toolbar, table headers, status bar |
| surface-raised | `#1a1d24` | Active tab, secondary buttons, badges |

**Colors — lines**
`#1e222a` inner hairline · `#24282f` primary border/divider · `#2e333c` control border · `#14161b` table row rule · `#101216` settings row rule

**Colors — text**
`#e6e8ec` primary · `#c7ccd6` secondary · `#9aa1ad` tertiary · `#8b93a1` muted · `#7f8794` terminal output · `#6b7280` dim/labels · `#4b525e` faintest (perms, timestamps, hints)

**Colors — accent & semantic**
| Token | Hex | Meaning |
|---|---|---|
| accent | `#ff6b3d` | Primary — active indicators, focus, upload, prompt, cursor |
| accent-light | `#ff8b5f` | Accent text/icon on dark, upload glyph, HTML badge |
| accent-ink | `#1a0d07` | Text on solid accent |
| accent-wash | `rgba(255,107,61,.08–.14)` | Outlined button fill, selection, active nav |
| success | `#3ecfb2` | Secure, verified, connected, complete |
| warning | `#ffb020` | Queued, skipped, unencrypted protocol, new host key |
| danger | `#ff4d4f` / text `#ff7a7c` | Failed transfers, secret files |
| info | `#7aa2ff` / `#62b6ff` | Folders, downloads, stylesheets |
| violet | `#b39bff` | S3/object storage, PHP |
| amber | `#ffc740` | JavaScript |

**Typography** — two families only:
- **Barlow** (400 / 500 / 600 / 700) — everything except the terminal. Google Fonts.
- **IBM Plex Mono** (400) — terminal drawer output only.

Scale in use: 15px/700 wordmark · 13px/600 dialog title · 12px/500 setting label & tree row · 12px/400 filename · 11.5px/600 button · 11.5px/400 field value · 11px/500 tab · 11px/400 table cell & help text · 10.5px/400 status bar · 9.5px/600 section label (`letter-spacing: .06–.08em`, uppercase) · 9px/600 state badge · 8.5px/600 extension badge. Terminal: 11.5px / 1.55.

Anything below 11px is uppercase, letter-spaced, and used only for labels and badges — never for reading text.

**Spacing** — 2 / 4 / 6 / 8 / 9 / 10 / 12 / 14 / 16 / 18 / 20px. Row heights: 25 (file) / 26 (table header, status bar) / 27 (tree, history) / 28 (control) / 30 (tab bar, nav) / 32 (queue row) / 34 / 36 (pane header) / 40 (title bar) / 46 (toolbar, footer).

**Radii** — 2px extension badge · 3px small badge · 4px chip · 5px control/button · 9px popover · 10px window · 9px pill (toggle track) · 50% dots and knobs.

**Shadows** — window `0 24px 60px rgba(0,0,0,.6)` · popover `0 20px 46px rgba(0,0,0,.6)`. No other elevation.

**Motion** — only two keyframes: `spin` (360°, 900ms, linear, infinite) on the connecting spinner; `pulse` (opacity .35 → 1 → .35, 1.1s, ease-in-out, infinite) on the terminal cursor and the focused-field caret.

## Assets
- **No real assets.** Everything is drawn in markup.
- **Icons**: nine placeholder inline SVGs at 11–16px, 1.3–1.6 stroke, `currentColor`-adjacent fills. Replace with a real set.
- **Logo**: placeholder blade glyph (two paths, a skewed bar plus a highlight facet). Replace with the real mark.
- **Fonts**: Barlow and IBM Plex Mono, loaded from Google Fonts. Bundle them locally for a desktop app.
- **No design system was available** — the attached design-system project was empty, so this palette and type scale were originated for xsaber. If a real xsaber brand system exists, treat this document's colors as replaceable and keep the structure, density, and semantic color roles.

## Files
- `xsaber UI.dc.html` — all five screens. A single HTML file: inline styles throughout, plus a `class Component` block at the end holding the sample data (file listings, site tree, queue, history, settings). Open it directly in a browser. The screens are laid out on a pan/zoom canvas; each is wrapped in an element with id `1a`–`1e` and a visible badge label.
- `README.md` — this document.
