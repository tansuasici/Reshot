<p align="center">
  <img src="assets/brand/reshot-1024.png" width="128" height="128" alt="Reshot logo">
</p>

<h1 align="center">Reshot</h1>

<p align="center">
  A fast, simple, open-source screenshot tool for macOS, Windows and Linux.<br>
  Press a hotkey, select an area, annotate, copy or save. That's it.
</p>

---

Reshot follows the spirit of Lightshot, which is loved for its speed and
simplicity but no longer maintained, while borrowing polish from tools like
Shottr and CleanShot. It lives in your menu bar / system tray, stays out of
the way, and never touches the network.

## Features

- **One hotkey** – <kbd>⌘</kbd> <kbd>⇧</kbd> <kbd>9</kbd> on macOS,
  <kbd>Ctrl</kbd> <kbd>Shift</kbd> <kbd>9</kbd> on Windows/Linux, or the tray menu.
- **Frozen screen** – the monitor under the cursor is captured instantly, so
  menus and hover states stay put while you select.
- **Region selection** with resize handles and a live pixel-size label.
- **Annotations** – pen, line, arrow, rectangle, marker and text, in eight colours, with undo.
- **Copy or save** – to the clipboard, or as PNG/JPEG.
- **HiDPI-correct** – output is cropped from the original pixels, at full Retina resolution.
- **Multi-monitor** aware, works above the macOS menu bar and over full-screen apps.
- **Local-only** – no accounts, no uploads, no telemetry.

## Keyboard shortcuts

| Key | Action |
|-----|--------|
| <kbd>⌘/Ctrl</kbd> <kbd>⇧</kbd> <kbd>9</kbd> | Take a screenshot |
| Drag | Select an area |
| <kbd>Enter</kbd> / <kbd>⌘/Ctrl</kbd> <kbd>C</kbd> | Copy (whole screen if nothing is selected) |
| <kbd>⌘/Ctrl</kbd> <kbd>S</kbd> | Save as… |
| <kbd>⌘/Ctrl</kbd> <kbd>Z</kbd> | Undo |
| <kbd>P</kbd> <kbd>L</kbd> <kbd>A</kbd> <kbd>R</kbd> <kbd>M</kbd> <kbd>T</kbd> | Pen, Line, Arrow, Rectangle, Marker, Text |
| <kbd>Esc</kbd> | Cancel |

## Install

Pre-built installers will be published on the
[Releases](https://github.com/tansuasici/Reshot/releases) page.

- **macOS**: on first use, allow Reshot under *System Settings → Privacy &
  Security → Screen Recording*. Builds are not notarized yet, so you may need
  to right-click → *Open* the first time.
- **Linux**: X11 and Wayland (via the screenshot portal) are supported by the
  capture library; tray support depends on your desktop environment.

## Build from source

Requirements: [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) 20+
and the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/tansuasici/Reshot.git
cd Reshot
npm install
npm run tauri dev      # run in development
npm run tauri build    # produce an installer in src-tauri/target/release/bundle
```

The overlay UI can also be developed in a plain browser with `npx vite`:
`src/ipc.ts` serves a generated test frame when it is not running inside Tauri.

## Tech

[Tauri 2](https://tauri.app/) with a Rust backend
([xcap](https://github.com/nashaofu/xcap) for capture,
[arboard](https://github.com/1Password/arboard) for the clipboard) and a
framework-free TypeScript + Canvas 2D overlay. See
[CODEBASE_MAP.md](CODEBASE_MAP.md) for the architecture.

## Contributing

Bug reports, ideas and pull requests are welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md).

## License

[MIT](LICENSE) © tansuasici
