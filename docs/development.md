# Development guide

## Prerequisites

- Node 22 or newer
- pnpm (the repo pins the version in `package.json`)
- Rust (stable toolchain) and the Tauri 2 system dependencies for your platform
- ffmpeg on your PATH. The app needs it at runtime and checks for it on first launch.
  - macOS: `brew install ffmpeg`
  - Windows: `winget install --id Gyan.FFmpeg -e`
  - Linux: `sudo apt install ffmpeg`

  The media tests use the bundled `ffmpeg-static` that `pnpm install` fetches, and skip if it is missing.

Install dependencies once from the repo root:

```
pnpm install
```

## Run the desktop app

```
pnpm dev:desktop
```

This starts Vite on port 1420 and the Tauri shell. The first Rust build takes a while. The desktop app keeps all data and built-in styles locally on this computer (your AI agent still needs its own internet connection).

To check everything:

```
pnpm test
pnpm typecheck
cargo test --manifest-path apps/desktop/src-tauri/Cargo.toml
```

## Where data lives

- App data (SQLite, `listune.db`) is in the operating system's app data directory for Listune Studio.
- `pnpm dev:desktop` runs as **Listune Studio Dev** (`app.listune.studio.dev`, from `src-tauri/tauri.dev.conf.json`), so it has its own app data and runs beside an installed Listune Studio (`app.listune.studio`) without sharing chats, settings or its window. On macOS the two folders are `~/Library/Application Support/app.listune.studio.dev` and `.../app.listune.studio`.
- Projects live in the working folder the user picks. Agent sessions run with the project folder, or a `.listune/<purpose>` subfolder of the working folder, as their working directory. They never run outside it.

## Permissions and the agent's own settings

Listune answers the ACP permission requests an agent sends (`session/request_permission`) and only ever picks the allow-once option after the user presses Allow. It can't see or block commands the agent runs without asking: if the agent's own configuration auto-allows a command (for example Claude Code's `permissions.allow` rules in `~/.claude/settings.json`, or a "yolo"/auto-approve mode), that command runs without a card in Listune. Listune does not change agent configuration, with one exception: Auto mode. Style studies always run in auto mode, and chats do when Settings › Agents › Auto mode for chats (or a chat's Auto toggle) is on. For Claude Code, Listune sends `session/set_mode` with `auto` (and `default` when turned off); if the agent doesn't list an `auto` mode or the switch fails, and for every other agent, Listune answers each permission request with its allow-once option (never allow-always) and shows "Ran without asking: <command>". A request with no allow-once option still shows the card. The setup (install) chat never runs in auto mode. The product copy is worded to match: "Listune shows you every command your agent asks to run, and nothing it asks about runs until you allow it."

## Release smoke test

Run this on a packaged build (`pnpm --filter @listune/desktop tauri build`) before shipping. A release build never falls back to the repo's copy of the agent plugin.

- [ ] The bundle has the plugin: `agent-plugin/.claude-plugin/plugin.json` is in the app's resources (macOS: `Listune Studio.app/Contents/Resources/agent-plugin/`). Start the app's binary from a terminal (macOS: `Listune Studio.app/Contents/MacOS/`), start a Claude chat and check stderr has no "plugin.json is missing" warning. Without the plugin Claude still gets the skills, as text in its first prompt, but the bundle is wrong.
- [ ] CSP: built-in style previews load on the Style screen (from the preview origins in `img-src`), the style popup plays its preview video (preview origins in `media-src`), and a rendered video and its frames play in the preview.
- [ ] Questions: ask an agent for topic ideas; its question (Claude's AskUserQuestion, opencode's question, Antigravity's ask_question) shows as a card with options and Other. Picking one starts the video in a new chat (the agent's POST /video).
- [ ] Windows: build and install the Windows bundle, run onboarding (winget installs), start a chat and render one video.

## CI and releases

CI runs only when a version tag (`v*`) is pushed, nothing runs on ordinary pushes or pull requests. `.github/workflows/release.yml` runs typecheck, the JS tests, clippy and the Rust tests, then builds macOS (Apple silicon and Intel), Windows and Linux into a **draft** GitHub release with signed updater bundles and `latest.json`. Before tagging, run the checks locally (`pnpm typecheck && pnpm test`, and `cargo clippy --all-targets -- -D warnings && cargo test` in `apps/desktop/src-tauri`).

To ship a version:

1. `pnpm version:set 0.2.0` (writes the version into both `package.json` files, `tauri.conf.json`, `Cargo.toml` and `Cargo.lock`).
2. Commit, then `git tag v0.2.0 && git push origin main v0.2.0`. The workflow fails early if the tag and the app version differ.
3. Check the draft release, then publish it. Publishing is what ships the update.

### Auto-update

Release builds check `https://github.com/lrmn7/listune-studio/releases/latest/download/latest.json` 10 seconds after launch and every 6 hours. When a newer version is there, the sidebar offers it; the download is checked against the public key in `tauri.conf.json` and installed on restart. Dev builds never check. Drafts and pre-releases are never "latest", so they reach nobody until published. This only works while the repository is public: release downloads from a private repository need a login.

The updater signing key is `~/.tauri/listune-studio.key` on the maintainer's machine, with its password in `~/.tauri/listune-studio.key.password`, and both are in the repository secrets (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). Back them up somewhere safe. If the key is lost, installed copies can never update again, and everyone has to reinstall by hand.

Installers are not code-signed yet. macOS shows them as from an unidentified developer (right-click › Open the first time), and Windows SmartScreen warns. On macOS an unsigned app also asks for the keychain password again after each update, because the keychain trusts one exact build. A Developer ID certificate and notarization fix both (add the `APPLE_*` secrets that tauri-action reads).

## Downgrades

Chats saved by this version can store `thought` parts (the agent's reasoning) and images the agent made (`file` parts with `modifiedAt`). An older build can't read a `thought` part, so going back to it breaks those chats' message lists. Don't ship a downgrade path.

## Notifications and background mode

- Notifications use `tauri-plugin-notification`. On desktop the plugin always reports the permission as granted, so Listune cannot tell when a user has turned its notifications off in the operating system (macOS, Windows). Settings › Agents has a "Send a test notification" button so people can check that notifications reach them.
- Background mode shows a tray icon only while it is on. On Linux the tray needs AppIndicator: the deb depends on `libayatana-appindicator3-1` and the rpm on `libayatana-appindicator-gtk3`; an AppImage uses the system's library, so it must be installed there. Without it the tray library crashes while loading; Listune catches that (`std::panic::catch_unwind`), turns background mode and Open at login back off, shows "Background mode needs a system tray, which isn't available here" in Settings, and never hides the window. The same failure at startup turns the setting off and the app opens normally.
- Launching Listune a second time shows the running window (`tauri-plugin-single-instance`). "Open at login" starts the app with `--hidden`, which keeps the window hidden when a tray icon exists.
- Dev builds register the login item for the dev binary, and notifications in dev may carry the terminal's identity until the app is bundled.
