# CLAUDE.project.md — Project Overlay

> This file is never modified by kit upgrades. Add project-specific rules here.
> Rules in this file override kit defaults when there's a conflict.

---

## Project Context
- **Reshot** is an open-source, cross-platform (macOS / Windows / Linux) screenshot tool in the spirit of Lightshot: hotkey → freeze the screen → select a region → annotate → copy / save. Speed and simplicity beat features.
- Stack: **Tauri 2** (Rust backend) + **vanilla TypeScript + Vite + Canvas 2D** frontend. No UI framework.
- Layout: the Cargo crate lives in `src-tauri/` (there is **no** root `Cargo.toml`). The frontend lives at the repo root (`index.html`, `src/`, `package.json`).
- Run all cargo commands from `src-tauri/` (`cd src-tauri && cargo …`). Canonical commands are declared in `.claude/commands.json`.
- Dev loop: `npm run tauri dev`. Frontend-only check: `npx tsc --noEmit`.
- **Backlog lives in Notion** (Reshot Board, data source `collection://41b67ebb-3867-815f-85f2-000bbe8ef5ff`, Durum: Backlog / Yapılacak / Devam Ediyor / İncelemede / Tamamlandı). `tasks/todo.md` only holds the plan for the current session's work. Move a card to *Tamamlandı* only after it is actually verified on a real machine.
- The user communicates in Turkish; code, comments and commits are in English.

---

## Additional Agent Docs
- Architecture overview → `CODEBASE_MAP.md`

---

## Project-Specific Protected Changes
- `src-tauri/tauri.conf.json` security settings (CSP, capabilities in `src-tauri/capabilities/`)
- `package.json` / `package-lock.json` dependency changes
- Global shortcut defaults (user muscle memory)
- Anything that sends data off the machine (Reshot is local-only; uploading is a future opt-in feature)

---

## Stack-Specific Rules
- **Capture path must stay fast**: hotkey → overlay visible target < 150 ms. Send raw RGBA over IPC (`tauri::ipc::Response`), never PNG-encode on the hot path.
- Pre-create the overlay window hidden; show/hide it, don't create/destroy per capture.
- HiDPI: the frontend works in CSS pixels; crop/export in **physical** pixels from the original capture so output is 1:1 with the screen.
- Platform-specific code goes behind `#[cfg(target_os = "…")]` in its own module (`src-tauri/src/platform/`); keep the shared path platform-agnostic.
- A failed global shortcut registration must never panic — log it and keep the tray working.
- Frontend: all `invoke` calls go through one module (`src/ipc.ts`) so the overlay can run in a plain browser with fixture data.
