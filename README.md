<p align="center">
  <img src="assets/brand/reshot-1024.png" width="128" height="128" alt="Reshot logo">
</p>

<h1 align="center">Reshot</h1>

<p align="center">
  <strong>Screenshot, mark it up, share it. In about two seconds.</strong><br>
  A fast, free and open-source screenshot tool for macOS, Windows and Linux.
</p>

<p align="center">
  <a href="https://github.com/tansuasici/Reshot/releases/latest"><img alt="Latest release" src="https://img.shields.io/github/v/release/tansuasici/Reshot?label=download&color=0A84FF"></a>
  <img alt="Platforms" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-232B3D">
  <a href="LICENSE"><img alt="MIT license" src="https://img.shields.io/badge/license-MIT-232B3D"></a>
</p>

---

Taking a screenshot shouldn't mean opening an editor, waiting for a window and
hunting for the export button. With Reshot you press one shortcut, drag over
what you want, add an arrow or a note if you need one, and press
<kbd>Enter</kbd>. The image is on your clipboard and Reshot is out of the way.

It keeps the speed and simplicity that made Lightshot popular, adds the polish
of tools like Shottr and CleanShot, and runs on every desktop. It is
**local-only**: no account, no cloud upload and no telemetry.

## Why Reshot

- ⚡ **Instant.** The screen under your cursor freezes as soon as you press
  the shortcut, so menus, tooltips and hover states stay put while you select.
- 🎯 **Precise.** Drag a region, fine-tune it with resize handles and see the
  exact pixel size as you go.
- ✏️ **Mark it up in place.** Pen, line, arrow, rectangle, highlighter and
  text, in eight colours, with undo. Nothing else to open.
- 📋 **Copy or save in one key.** <kbd>Enter</kbd> copies and
  <kbd>⌘/Ctrl</kbd>&nbsp;<kbd>S</kbd> saves as PNG or JPEG. The save dialog
  remembers the last folder you used.
- 🖥️ **Sharp everywhere.** Output is cut from the original pixels at full
  Retina/HiDPI resolution, on any monitor of a multi-monitor setup.
- 🪶 **Lightweight.** About a 4 MB download. It lives in your menu bar or system
  tray, and opening the app starts a capture right away.
- 🔒 **Private.** Your screenshots never leave your computer.

## Download

Get the latest version from the
**[Releases page](https://github.com/tansuasici/Reshot/releases/latest)**.

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `Reshot_x.y.z_aarch64.dmg` |
| macOS (Intel) | `Reshot_x.y.z_x64.dmg` |
| Windows | `Reshot_x.y.z_x64-setup.exe` or `.msi` |
| Linux | `.AppImage`, `.deb` or `.rpm` |

### First launch

Reshot is not code-signed by Apple or Microsoft yet, so your system will ask
you to confirm the first time you open it.

- **macOS**
  1. Open the `.dmg` and drag **Reshot** into **Applications**.
  2. Open Reshot. If macOS says it can't verify the developer, go to
     *System Settings → Privacy & Security* and click **Open Anyway**.
  3. When macOS asks for **Screen Recording** permission, allow it, then quit
     and reopen Reshot.
- **Windows:** if SmartScreen appears, click **More info → Run anyway**.
- **Linux:** works on X11 and on Wayland (through the desktop's screenshot
  portal). Whether the tray icon appears depends on your desktop environment.

## How to use it

1. Press <kbd>⌘</kbd>&nbsp;<kbd>⇧</kbd>&nbsp;<kbd>9</kbd> on macOS or
   <kbd>Ctrl</kbd>&nbsp;<kbd>Shift</kbd>&nbsp;<kbd>9</kbd> on Windows and Linux.
   You can also click the Reshot icon in the tray or Dock.
2. Drag over the area you want.
3. Mark it up if you need to, then press <kbd>Enter</kbd> to copy or
   <kbd>⌘/Ctrl</kbd>&nbsp;<kbd>S</kbd> to save.

### Keyboard shortcuts

| Key | Action |
|-----|--------|
| <kbd>⌘/Ctrl</kbd> <kbd>⇧</kbd> <kbd>9</kbd> | Take a screenshot |
| Drag | Select an area |
| <kbd>Enter</kbd> / <kbd>⌘/Ctrl</kbd> <kbd>C</kbd> | Copy (the whole screen if nothing is selected) |
| <kbd>⌘/Ctrl</kbd> <kbd>S</kbd> | Save as PNG or JPEG |
| <kbd>⌘/Ctrl</kbd> <kbd>Z</kbd> | Undo |
| <kbd>P</kbd> <kbd>L</kbd> <kbd>A</kbd> <kbd>R</kbd> <kbd>M</kbd> <kbd>T</kbd> | Pen, line, arrow, rectangle, marker, text |
| <kbd>Esc</kbd> | Cancel |

## Build from source

You need [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/)
20 or newer, and the
[Tauri prerequisites](https://tauri.app/start/prerequisites/) for your OS.

```bash
git clone https://github.com/tansuasici/Reshot.git
cd Reshot
npm install
npm run tauri dev      # run in development
npm run tauri build    # build installers into src-tauri/target/release/bundle
```

You can also work on the overlay UI in a plain browser with `npx vite`. When
it isn't running inside Tauri, `src/ipc.ts` serves a generated test frame.

Releases are built by GitHub Actions: pushing a `v*` tag builds installers for
every platform and attaches them to a GitHub release.

## Under the hood

Reshot is built with [Tauri 2](https://tauri.app/). The Rust backend uses
[xcap](https://github.com/nashaofu/xcap) for screen capture and
[arboard](https://github.com/1Password/arboard) for the clipboard. The
overlay is framework-free TypeScript on Canvas 2D, and pixels go to and from
it as raw bytes, with no encoding on the hot path.
[CODEBASE_MAP.md](CODEBASE_MAP.md) describes the architecture.

## Contributing

Bug reports, ideas and pull requests are all welcome. See
[CONTRIBUTING.md](CONTRIBUTING.md) to get started.

## License

[MIT](LICENSE) © tansuasici
