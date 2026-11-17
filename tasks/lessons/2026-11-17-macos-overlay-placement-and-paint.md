---
title: macOS overlay shifted on first capture and flickered on every show
created: 2026-11-17
updated: 2026-11-17
tags: [macos, tauri, overlay, webkit]
problem_type: bug
source: correction
confidence: high
top_rule: true
status: active
related: []
supersedes: []
applies_to: [overlay, macos]
contradicts: []
related_decisions: []
---

## Issue

1. The first capture on a monitor (or the first after switching monitors) drew
   the frozen frame shifted vertically by the height difference between the old
   and new window size (up when growing from 800x600, down by 98 pt when
   shrinking 1920x1080 -> 1512x982).
2. Every capture flickered: a blank/stale frame flashed before the new one.

## Root Cause

1. On macOS `NSWindow` resizes keep the bottom-left corner fixed. `place_window`
   called `set_position` before `set_size`, so the resize moved the top edge.
2. WebKit does not paint a hidden window. The frame drawn while hidden was only
   composited after `show()`, so the previous content was visible for a frame.
   Waiting for rAF before *hiding* does not help.

## Rule

- When placing a window: set the size first, then the position.
- To show a webview with fresh content and no flash: show it at alpha 0, wait
  for two rAFs (with a timeout fallback), then set alpha 1
  (`show_overlay` -> `nextPaint()` -> `reveal_overlay`).

## Verification

- `src-tauri/src/platform/mod.rs::place_window` calls `set_size` before `set_position`.
- Manual: Cmd+Shift+9 repeatedly on both monitors, alternating — no shift, no flash.

## References

- `src-tauri/src/platform/macos.rs` (`set_overlay_opaque`), `src/main.ts` (`onCapture`).
