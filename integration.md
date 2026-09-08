# xsaber integration

Status: 2026-09-08. xsaber is a new consumer of xsync v3 and a planned port of
selected excalibur engine patterns. This document records the seams and their
current workarounds; it does not claim that a provider prerequisite is done.

Repository ownership is split deliberately: E0-S1 creates and publishes the
xsaber `xcalibur` branch and proves its local/CI checks; E17-S1 registers that
published commit in the suite and moves the superproject pin. A local checkout
can therefore build before it is a root submodule.

## xsync v3 provider seam

The primary connection is one russh SSH session. xsaber opens
`channel.exec("xs --server <root>")`, converts the channel into an async
stream, and calls `xsync_client::Client::from_stream`. The empty export name is
used for the v3 mount. If the remote `xs` binary is unavailable, the session
degrades to `russh-sftp`; the UI protocol badge and capabilities must reflect
that choice.

The `E17-S2` plan names six provider capabilities. Four map to current xsync
backlog IDs; the two capability gaps whose earlier baseline stories are already
complete do not yet have distinct open provider IDs. Those missing IDs are
called out explicitly below. `E17-S2` cannot close until xsync records follow-up
stories for them.

| xsync story or tracking gap | Consumer need | Current xsaber behavior | When it lands |
|---|---|---|---|
| `E3-S4` — cancellation and timeouts | Cancel an in-flight read, listing, or transfer request. | Cancel the local operation and close/abandon only the request handle; do not claim a wire cancellation. | Use the client `Cancel(related_id)` and map the terminal cancellation response to the job. |
| `E5-S1b` — owner and group names | Render owner/group columns when the server can resolve them. | Keep numeric uid/gid or an unknown value; never invent a name. | Request the `OWNER_NAMES` feature and show names only when advertised. |
| `E2-S2` — SSH transport and bring-your-own stream | Reuse xsaber's authenticated russh channel without a second SSH process. | Use the documented `from_stream` path; remote-binary selection stays an argv element. | Track the provider compatibility matrix and use its finalized recipe/override contract. |
| **Missing provider ID — follow-up to completed `E5-S4`** | Native recursive delete and parent-creating directory operations. | The completed `E5-S4` baseline covers the ordinary mutation set, not these new server-side verbs (see `xsync/legacy/xsyncv3.md`). xsaber walks and deletes client-side and issues one-level `mkdir` calls for `mkdir -p`, preserving progress and cancellation. | xsync must assign an open follow-up story and capability-gated verbs; retain the loop for older servers. |
| `E7-S1` (with `E7-S2`/`E7-S3`) — directory watch, overflow and backend limits | Live refresh of an open pane. | Poll according to `dir_cache_ms`; an overflow or uncertain result forces a full relist. | Subscribe to `Watch`/`WatchEvent` and rescan on `WatchOverflow`, respecting advertised limits. |
| **Missing provider ID — follow-up to completed `E5-S2`** | Browse directories too large for one path listing. | The completed `E5-S2` baseline provides handle-based directory reading and stable cursors, but not a paged `read_dir_path` overflow variant (see `xsync/legacy/xsyncv3.md`). xsaber uses the handle path when `read_dir_path` overflows. | xsync must assign an open follow-up story for the paged path API; retain the handle fallback until then. |

The client must feature-gate every message. The current server grants
`STAGE_RESUME | PATH_LISTING | SUBTREE_LISTING | PATH_READ`; unknown message
types terminate the session, so xsaber must not infer support from a desired
UI control. `xsync-client` currently has no cancel, watch, recursive delete,
`mkdir -p`, owner/group names, or paged path-overflow API.

The two missing provider IDs above are deliberate tracking gaps, not renamed
stories. The `E17-S2` acceptance criterion remains open until xsync's backlog
assigns and scopes both follow-ups.

## Excalibur engine port

xsaber ports the following patterns from `../excalibur/crates/engine` with
attribution: `error.rs`'s `Surface` taxonomy, `hostkeys/`, `secrets/`, and the
broadcast transfer-queue shape. It does not path-depend on excalibur. The
source is unversioned in the current suite checkout, so E17-S4 must record the
source commit and add a drift script before this port is treated as stable.

The port boundary is deliberate: xsaber's `crates/engine` owns sessions,
remote filesystem rules, transfer policy and secrets; `crates/app` owns GPUI
composition and UI prompts. No GPUI types cross into the engine.

## Shared-crate question

xsaber is the third GUI, which opens the suite question in `E17-S3`: whether to
extract progress/event DTOs, remote filesystem traits, host-key handling and
secret wrappers into a shared crate. Until a dated suite decision exists,
xsaber keeps its own engine types and a documented port from excalibur. Do not
silently create a shared dependency as part of an xsaber feature.
