# CODEBASE_MAP.md

## What

Reshot is an open-source, cross-platform (macOS / Windows / Linux) screenshot tool in the spirit of Lightshot. It lives in the tray/menu bar. Press the hotkey and it freezes the monitor under the cursor; you select a region, annotate it, then copy it to the clipboard or save it as PNG/JPG.

## Why

Lightshot is the favourite, but it is no longer maintained. Shottr and CleanShot are macOS-only and paid. Reshot aims to be fast, simple and free on all three platforms.

**Core constraints**
- Hotkey to frozen overlay in under ~150 ms.
- Pixel-exact (HiDPI-correct) output.
- Local-only, with no network calls.

## Tech Stack

- **Shell**: Tauri 2 (pinned `2`; do not jump to the 3.x alphas).
- **Backend**: Rust, edition 2024, crate in `src-tauri/`.
  - `xcap`: capture.
  - `arboard`: clipboard.
  - `image`: PNG/JPEG encoding.
  - `tauri-plugin-global-shortcut`, `tauri-plugin-dialog`, `tauri-plugin-single-instance`.
  - `objc2` + `objc2-app-kit` (macOS only): window level and Spaces behaviour.
- **Frontend**: vanilla TypeScript, Vite and Canvas 2D. No UI framework.
- **Error handling**: a hand-written error enum in `src-tauri/src/error.rs`, serialised to a string for the frontend.
- **Testing**: `cargo test` for the pure logic (geometry, cropping, encoding).

## Key Commands

The canonical commands are also declared in `.claude/commands.json`.

| Action | Command |
|--------|---------|
| Dev (full app) | `npm run tauri dev` |
| Release bundle | `npm run tauri build` |
| Frontend typecheck | `npx tsc --noEmit` |
| Rust lint | `cd src-tauri && cargo clippy --all-targets -- -D warnings` |
| Rust test | `cd src-tauri && cargo test` |
| Format | `cd src-tauri && cargo fmt` |
| Capture probe | `cd src-tauri && cargo run --release --example capture_probe -- <out-dir>` |

## Directory Structure

```text
.
├── index.html              # overlay window markup
├── settings.html           # Settings window markup (second Vite entry)
├── src/                    # frontend (overlay UI)
│   ├── main.ts             # boot, selection, toolbars, keyboard, export (crop + annotations)
│   ├── ipc.ts              # the ONLY place that talks to Tauri (browser fixture fallback)
│   ├── settings.ts/.css    # Settings window: shortcut recorder, folder, format, open at login
│   ├── accelerator.ts      # KeyboardEvent → Tauri accelerator string + display labels
│   ├── geometry.ts         # rects, resize handles, hit-testing
│   ├── annotations.ts      # annotation model + drawing (pen, line, arrow, rect, marker, text)
│   ├── icons.ts            # inline SVG toolbar icons
│   └── styles.css
├── src-tauri/
│   ├── Cargo.toml          # crate manifest (protected)
│   ├── tauri.conf.json     # app + overlay window config (protected)
│   ├── capabilities/       # IPC permissions for the overlay window (protected)
│   ├── examples/capture_probe.rs
│   └── src/
│       ├── main.rs         # calls reshot_lib::run()
│       ├── lib.rs          # builder, plugins, setup, command registration
│       ├── capture.rs      # capture the monitor under the cursor, hold the frame in state
│       ├── commands.rs     # IPC commands: frame pixels, copy, save, close, settings
│       ├── settings.rs     # Settings struct + settings.json store (migrates v0.1 last-save-dir)
│       ├── settings_window.rs # opens/focuses the Settings webview
│       ├── tray.rs         # tray icon + menu (Capture / Settings… / Quit)
│       ├── shortcut.rs     # global hotkey: register, suspend/resume, change at runtime
│       ├── overlay.rs      # position/show/hide the pre-created overlay window
│       ├── error.rs        # Error enum
│       └── platform/       # #[cfg] per-OS tweaks (macos.rs: window level, Spaces, alpha)
├── assets/brand/           # logo.svg, tray.svg (menu bar template), 1024px app icon
├── .github/                # CI (fmt/clippy/test on 3 OSes, tsc) + tag-triggered release
└── tasks/                  # session plan (backlog lives in Notion)
```

## Architecture

1. **Hotkey or tray** calls `capture::capture_under_cursor()`.
   - Picks the monitor that contains the cursor.
   - Grabs it with xcap as RGBA.
   - Stores the frame (pixels plus the monitor's position, size and scale) in managed state.
2. **Overlay** runs `overlay::place(rect)`.
   - Resizes, then moves (size first: macOS resizes keep the bottom-left corner fixed) the hidden, pre-created, borderless overlay window to cover that monitor.
   - Emits `capture-ready` carrying the frame metadata (physical and CSS size).
3. **Frontend** loads the frame and lets the user edit it.
   - Pulls the raw RGBA through `get_frame`, a raw `ipc::Response` with no PNG encode, and paints it to a full-screen canvas.
   - Waits until the viewport has the new size, paints, then calls `show_overlay` (shown at alpha 0) and, after two animation frames, `reveal_overlay` (alpha 1). WebKit does not paint hidden windows, so this avoids a stale-frame flash.
   - The user selects a region and annotates it, in CSS pixels.
4. **Export** happens in the frontend, then in Rust.
   - The frontend crops the original frame in physical pixels and replays the annotations scaled by the device pixel ratio.
   - It sends the RGBA to `copy_image` (arboard, no encode) or `save_image` (save dialog, then PNG/JPEG encode).
   - Then it calls `close_overlay`, which hides the window and drops the frame.

## Critical Files

| File | Purpose |
|------|---------|
| `src-tauri/src/capture.rs` | Hot path. Keep it allocation-light, with no encoding. |
| `src-tauri/src/platform/macos.rs` | Overlay above the menu bar, on all Spaces and over full-screen apps. |
| `src/ipc.ts` | The single IPC boundary. Swap it for fixtures to run the UI in a browser. |
| `src/main.ts` | Overlay UI and export: HiDPI correctness of the output. |
| `src-tauri/src/platform/mod.rs` | Coordinate spaces (xcap points vs physical) and window placement. |
