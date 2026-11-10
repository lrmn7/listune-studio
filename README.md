# Listune Studio

> A lightweight, local desktop studio for crafting motion graphics and short videos with code and your favourite AI agent.

Listune Studio pairs your local coding agent (Claude Code, opencode, Antigravity CLI, Codex, or any ACP agent) with Remotion to turn ideas, scripts, and data into motion videos right on your computer.

No complex timelines, no bloated cloud subscriptions, and zero tracking.

---

## Features

- **Bring Your Own Agent**: Connects seamlessly over ACP (Agent Control Protocol). Use the CLI agent you already trust.
- **28 Built-in Styles**: From *Atlas Journey* to *Swiss Motion* and *Data Storytelling* all previews and assets run 100% locally.
- **You Stay in Control**: Every command your agent wants to run asks for your permission first (unless you toggle Auto mode).
- **Fully Offline Core**: Projects and styles live in a local SQLite database on your disk. No telemetry, no background calls home.
- **Fast Preview**: Live frame-by-frame scrubbing and quick renders powered by Remotion + Webview.

---

## Quick Install

### macOS & Linux
```bash
curl -fsSL https://studio.listune.app/install.sh | sh
```

### Windows (PowerShell)
```powershell
irm https://studio.listune.app/install.ps1 | iex
```

- **macOS:** Installs to `/Applications` (or `~/Applications`).
- **Linux:** Native `.deb`, `.rpm`, or standalone `AppImage`.
- **Windows:** Installs per-user, no admin prompt needed.

*Prefer manual downloads? Grab the latest installer directly from [Releases](https://github.com/lrmn7/listune-studio/releases).*

---

## Running From Source

If you want to hack on Listune Studio yourself:

### Prerequisites
- [Node.js](https://nodejs.org/) 22+
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) (latest stable)
- [FFmpeg](https://ffmpeg.org/) (for rendering & video clips)

### Start Developing
```bash
# 1. Install dependencies
pnpm install

# 2. Launch desktop app in dev mode
pnpm dev:desktop
```

For more details on platform dependencies and where app data lives, see [docs/development.md](docs/development.md).

### Useful Commands
| Command | What it does |
| --- | --- |
| `pnpm dev:desktop` | Runs Tauri + Vite in development mode |
| `pnpm test` | Runs unit tests across all workspace packages |
| `pnpm typecheck` | Checks TypeScript types without emitting |
| `pnpm --filter @listune/desktop build` | Builds the frontend for production |

---

## Project Structure

```
listune-studio/
├── apps/
│   └── desktop/        # Tauri (Rust) + React frontend
├── packages/
│   ├── shared/         # Shared TypeScript types & schemas
│   └── skills/         # Bundled agent skills & styles catalog
├── templates/          # Starter Remotion video templates
└── docs/               # Architecture and development docs
```

---

## Privacy & Local-First

- **Zero Telemetry**: We don't collect analytics, track user events, or phone home.
- **Local Storage**: Your projects, briefs, renders, and custom styles stay in your local working directory and SQLite database.
- **Transparent Permissions**: Every terminal command executed by an agent is displayed for your approval.
