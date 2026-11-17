# Task Board

Track current and upcoming tasks here. The agent updates this file as work progresses.

Give the task you are actively working on an `h3` heading under **In Progress** —
`session-start.sh` reads the first one and injects it as the session's active task.

---

## In Progress

### v0.2 — Settings window (Notion: "Ayarlar penceresi", P1) — in progress (plan approved)
**Acceptance:** from the tray, open Settings; change the capture shortcut, the
default save folder, the default format (PNG/JPEG) and "open at login"; every
change applies immediately and survives a restart, on macOS/Windows/Linux.

Design:
- Storage: `settings.json` in `app_config_dir`, a serde `Settings` struct with
  defaults (no store plugin — serde is already a dependency). Replaces the
  `last-save-dir` file (migrated on first read).
- Rust `settings` module: `load` / `save`, managed state; commands
  `get_settings`, `set_shortcut` (unregister old → register new → persist;
  returns an error if the combo is taken, old one stays), `set_save_dir`
  (folder picker), `set_format`, `set_autostart`.
- Window: second webview `settings` created on demand from a new tray item
  "Settings…" (focus it if already open). Vite multi-page: `settings.html` +
  `src/settings.ts`; own capability file with `core:default` only.
- Shortcut recorder: click field → press combo → JS builds a Tauri
  accelerator string → `set_shortcut`. Tray menu accelerator label updates.
- Format: the save dialog's suggested file name/filter order follows the setting.
- Open at login: `tauri-plugin-autostart` (**new dependency — needs approval**).

Steps:
- [x] Rust `settings` module + unit tests (defaults, round-trip, migration)
- [x] Commands + dynamic shortcut re-registration + tray item
- [x] Settings window (HTML/TS/CSS) + capability + vite multi-page
- [x] Autostart plugin wiring
- [x] Verify: fmt, clippy, check, test (13), release build, tsc, vite build; UI checked in browser
- [x] Smoke on macOS via tray (shortcut/folder/format/login item, restart persistence) — user
- [ ] PR + green CI on macOS/Windows/Linux

### MVP vertical slice (macOS first) — shipped as v0.1.0
Plan approved by the user ("Evet, başla"). Backlog/cards live in Notion.
- [x] Tauri 2 scaffold + capture probe (xcap works, 25–90 ms/monitor)
- [x] Claude Code Kit + project rules (`CLAUDE.project.md`, `CODEBASE_MAP.md`, `.claude/commands.json`)
- [x] Rust backend: tray, global hotkey, capture under cursor, overlay window, macOS level, copy/save/close commands
- [x] Overlay UI: selection + handles, tools (pen/line/arrow/rect/marker/text/color/undo), toolbars, keyboard shortcuts
- [ ] Build + test on macOS (hotkey, multi-monitor, HiDPI export size, speed)
- [ ] Open-source files: LICENSE (MIT), README, CONTRIBUTING, issue templates, CI + release workflow
- [ ] Update Notion cards (only verified items → Tamamlandı)

---

## Up Next

---

## Done

---

## Not Now

Parked scope — deferred work, revisit when prioritized. (CLAUDE.md → Scope Discipline routes out-of-scope items here.)

- Local `tauri build --bundles app` ends with "failed to run xattr: option -r not recognized" (a non-Apple `xattr` on PATH shadows `/usr/bin/xattr`); the .app is still produced. CI is unaffected.
