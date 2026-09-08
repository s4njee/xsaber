# ADR-0001: Use GPUI Kit for the desktop shell

- Date: 2026-09-08
- Status: Accepted

## Context

xsaber needs a native-feeling dual-pane desktop shell, tables, splitters,
menus, dialogs, trees and window chrome. The plan's research found that bare
GPUI exposes primitives but not the widget set, while the name “GPUIX” refers
to unrelated projects.

## Decision

Use GPUI through `gpui-kit = "0.6"`. Keep the app shell in `crates/app` and
keep the engine free of GPUI.

## Consequences

The project gets the required Table, DockArea, resizable panes, dialogs,
menus, Tree, inputs and native window components from one Rust widget layer.
The project must track GPUI Kit's API and build its own terminal, lazy file
tree behavior and cross-table drag where the kit has gaps.

## Alternatives rejected and evidence

- Tauri was rejected because the product is explicitly choosing a pure-Rust
  native shell rather than a webview; the existing Tauri GUIs remain separate.
- `remorses/gpuix` was rejected: research found React/Node bindings, not a
  Rust app API or the required table/tabs/splitter/terminal set.
- `AzureZee/gpuix` was rejected: it is a small mirror/re-export shim, not a
  maintained widget library.
- crates.io `gpuix` was rejected: docs.rs describes it as a reservation, not a
  usable library.
- Bare `gpui` was rejected: the plan's research found 12 primitives and no
  widgets; even a text input is about 780 lines in the examples.
