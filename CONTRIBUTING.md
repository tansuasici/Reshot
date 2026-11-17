# Contributing to Reshot

Thanks for helping! Reshot aims to stay **fast, simple and local-only** — please
keep that in mind when proposing features.

## Getting started

1. Install [Rust](https://rustup.rs/) (stable), Node.js 20+ and the
   [Tauri prerequisites](https://tauri.app/start/prerequisites/).
2. `npm install`
3. `npm run tauri dev`

To iterate on the overlay UI only, run `npx vite` and open it in a browser —
`src/ipc.ts` provides a fixture frame outside Tauri.

## Before opening a pull request

Run the same checks as CI:

```bash
npx tsc --noEmit
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Then try the change in the real app (`npm run tauri dev`), ideally on more than
one monitor / scale factor if it touches capture or the overlay.

## Guidelines

- Keep pull requests small and focused; one change per PR.
- Use [Conventional Commits](https://www.conventionalcommits.org/)
  (`feat:`, `fix:`, `docs:`, `refactor:`, `chore:` …).
- Rust: no `.unwrap()`/`.expect()` outside tests, errors go through
  `src-tauri/src/error.rs`. Any `unsafe` needs a `// SAFETY:` comment.
- All Tauri calls from the frontend go through `src/ipc.ts`.
- New dependencies need a short justification in the PR description.
- Platform-specific code lives in `src-tauri/src/platform/`.

## Reporting bugs

Please use the issue templates and include your OS, version, monitor setup
(count, resolution, scaling) and steps to reproduce. A screen recording helps a
lot for overlay/flicker issues.

By contributing you agree that your contributions are licensed under the
[MIT License](LICENSE).
