<div align="center">

# Pluely

### Lightning-fast, privacy-first AI assistant for your desktop.

[![Tauri](https://img.shields.io/badge/Built%20with-Tauri-6366f1?logo=tauri)](https://tauri.app/)
[![React](https://img.shields.io/badge/Frontend-React%20%2B%20TypeScript-3178C6?logo=react)](https://reactjs.org/)
[![Rust](https://img.shields.io/badge/Backend-Rust-000000?logo=rust)](https://rust-lang.org/)
[![License: GPL v3](https://img.shields.io/badge/License-GPL%20v3-blue.svg)](LICENSE)
[![GitHub](https://img.shields.io/badge/GitHub-rahulcvwebsitehosting/pluely--fixes-181717?logo=github)](https://github.com/rahulcvwebsitehosting/pluely-fixes)

**A translucent desktop overlay AI assistant that works during meetings, interviews, and conversations — invisible to everyone except you.**

</div>

---

## What This Is

Pluely is a **privacy-first AI assistant** that lives in a translucent overlay on your desktop. It sits above all applications, activated by a keyboard shortcut, and provides real-time AI assistance without anyone knowing you're using it.

This repository is a **bug-fix fork** of the original Pluely project, focused on resolving critical issues that affected usability across Windows, macOS, and Linux — including window focus problems, shortcut reliability, platform-specific rendering bugs, and integration with AI provider APIs.

> **What we fixed:** hide icon on Windows, Ctrl+Arrow / Ctrl+\\ toggle, macOS file attachment, Ollama Cloud API, OpenAI Responses API, browser fullscreen preservation, and more.

---

## Quick Overview

| Feature | Description |
|---------|-------------|
| **Invisible Overlay** | Translucent window sits above all apps — undetectable in screen shares, recordings, and video calls |
| **System Audio Capture** | Capture and transcribe meeting audio in real-time with voice activity detection |
| **Voice Input** | Speech-to-text via OpenAI Whisper, ElevenLabs, Groq, or custom STT providers |
| **Screenshot Capture** | Capture full screen or selected areas for AI visual analysis |
| **File Attachments** | Attach documents, images, and code files for AI context |
| **Any AI Provider** | BYO key — OpenAI, Anthropic, Google, Groq, Ollama, or custom endpoints via curl |
| **Custom STT Providers** | Wire any speech-to-text API with a curl-based configuration |
| **Custom System Prompts** | Define personas, response styles, and behavior profiles |
| **Full Keyboard Control** | Customize every shortcut — toggle, move, focus, record, screenshot |
| **Cross-Platform** | macOS, Windows, Linux — native performance on all three |

---

## Table of Contents

- [Fixes & Improvements](#fixes--improvements)
- [Installation & Setup](#installation--setup)
- [Build from Source](#build-from-source)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

---

## Fixes & Improvements

This fork applies the following bug fixes and enhancements to the original Pluely codebase:

| # | Issue | Fix |
|---|-------|-----|
| 1 | **Hide icon (Windows)** | Hide/show cycle after `set_skip_taskbar` so taskbar button is actually removed |
| 2 | **Ctrl+Arrow move window (Windows)** | Made overlay non-focusable (`WS_EX_NOACTIVATE`) so keyboard events route correctly |
| 3 | **Ctrl+\\ toggle (Windows)** | Same non-focusable overlay fix — toggle no longer steals OS focus |
| 4 | **File attachment (macOS)** | Replaced `className="hidden"` with offscreen clip rect (WKWebView ignores `.click()` on `display:none`) |
| 5 | **Ollama Cloud API** | Fixed inverted ternary causing CORS failures for external API calls |
| 6 | **OpenAI Responses API** | Added `openai-responses` provider with correct request/response paths |
| 7 | **Fullscreen preservation** | Overlay shows without stealing focus; input focus only activates on explicit user action |

---

## Installation & Setup

### Prerequisites

- **Node.js** v18+
- **Rust** (latest stable)
- **npm** or **yarn**

### Quick Start

```bash
# Clone the repository
git clone https://github.com/rahulcvwebsitehosting/pluely-fixes.git
cd pluely-fixes

# Install dependencies
npm install

# Start development server
npm run tauri dev
```

### Build for Production

```bash
npm run tauri build
```

Platform-specific installers are created in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg`
- **Windows**: `.msi`, `.exe`
- **Linux**: `.deb`, `.rpm`, `.AppImage`

---

## Key Features

### Invisible Overlay

The translucent overlay window sits seamlessly above all applications. It's undetectable in Zoom, Google Meet, Microsoft Teams, and Slack Huddles — invisible to your audience while you see it clearly. Screenshot-proof and recording-proof by design.

### System Audio Capture

Record and transcribe system audio in real-time with voice activity detection. Perfect for meetings, presentations, or any audio playing on your computer. The captured audio is processed through your selected STT provider and can be automatically analyzed by the AI.

**Keyboard Shortcut:** `Cmd+Shift+M` (macOS) / `Ctrl+Shift+M` (Windows/Linux)

### Voice Input

Convert speech to text using advanced STT providers — OpenAI Whisper, ElevenLabs, Groq Whisper, or custom providers. Voice activity detection automatically identifies when you're speaking.

**Keyboard Shortcut:** `Cmd+Shift+A` (macOS) / `Ctrl+Shift+A` (Windows/Linux)

### Screenshot Capture

Two modes:

- **Full Screen** — Capture the entire screen with one click
- **Selection Mode** — Click and drag to select a specific area

**Keyboard Shortcut:** `Cmd+Shift+S` (macOS) / `Ctrl+Shift+S` (Windows/Linux)

Processing modes:

- **Manual** — Screenshots are added to attachments; submit with your own prompt
- **Auto** — Automatically sent to AI with a configurable default prompt

### File Attachments

Attach documents, images, and code files for AI analysis. Supports multiple file types with visual indicators. Drag-and-drop or browse from your system.

### Custom AI Providers

Connect any LLM provider using simple curl commands — OpenAI, Anthropic, Google Gemini, xAI Grok, Mistral, Cohere, Perplexity, Groq, Ollama, or your own custom endpoint. Full streaming support.

Dynamic variables: `{{TEXT}}`, `{{IMAGE}}`, `{{SYSTEM_PROMPT}}`, `{{MODEL}}`, `{{API_KEY}}`.

### Custom STT Providers

Wire any speech-to-text API with curl-based configuration. Dynamic variables: `{{AUDIO}}`, `{{API_KEY}}`, `{{LANGUAGE}}`.

### Custom System Prompts

Create unlimited system prompts to control AI behavior — define personas, writing styles, response formats, and specialized knowledge domains. Switch between prompts instantly.

### Full Keyboard Control

Every action is customizable:

| Action | Default Shortcut |
|--------|-----------------|
| Toggle Dashboard | `Cmd+Shift+D` / `Ctrl+Shift+D` |
| Toggle Window | `Cmd+\` / `Ctrl+\` |
| Focus Input | `Cmd+Shift+I` / `Ctrl+Shift+I` |
| Move Window | Hold `Cmd` / `Ctrl` + arrows |
| System Audio | `Cmd+Shift+M` / `Ctrl+Shift+M` |
| Voice Input | `Cmd+Shift+A` / `Ctrl+Shift+A` |
| Screenshot | `Cmd+Shift+S` / `Ctrl+Shift+S` |

### Dashboard

Full dashboard with conversation history (searchable), AI provider configuration, system prompt management, app settings (theme, autostart, icon visibility, always-on-top), audio device selection, screenshot configuration, and response customization (length, language, auto-scroll).

---

## Architecture

```
pluely-fixes/
├── src-tauri/               # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs          # App entry point
│   │   ├── lib.rs           # Plugin registration, setup
│   │   ├── shortcuts.rs     # Global shortcut handling, window toggle
│   │   ├── window.rs        # Window management, positioning, focus
│   │   ├── capture.rs       # Screenshot capture overlay
│   │   ├── api.rs           # API routing for AI/STT calls
│   │   ├── activate.rs      # License activation
│   │   ├── speaker/         # System audio capture modules
│   │   └── db/              # SQLite database layer
│   ├── tauri.conf.json      # Tauri configuration
│   └── capabilities/        # Permission capabilities
│
├── src/                     # React TypeScript frontend
│   ├── hooks/               # Custom React hooks (window, shortcuts, etc.)
│   ├── pages/               # Route pages (app, chats, settings, dev, etc.)
│   ├── components/          # Reusable UI components
│   ├── config/              # Constants (AI providers, STT, shortcuts)
│   ├── lib/                 # Utilities, storage, functions
│   └── contexts/            # React contexts (app, theme)
│
└── package.json             # Dependencies and scripts
```

**Tech stack:**

| Layer | Technology |
|-------|-----------|
| Desktop framework | Tauri 2.8 |
| Frontend | React 18 + TypeScript |
| Styling | Tailwind CSS + shadcn/ui |
| Backend | Rust |
| Database | SQLite (via tauri-plugin-sql) |
| State | React hooks + localStorage |
| API transport | `fetch` (local) + tauri `http` plugin (external) |
| Shortcuts | tauri-plugin-global-shortcut |

---

## Contributing

This fork focuses on **bug fixes and stability improvements** for the original Pluely codebase.

- ✅ Bug-fix pull requests are welcome
- ✅ Improvements to existing functionality
- ❌ New features or large UI overhauls are outside scope

### How to Contribute

1. Fork the repository
2. Create a feature branch (`git checkout -b fix/your-fix`)
3. Commit your changes
4. Push to your branch
5. Open a Pull Request

---

## License

**GNU General Public License v3.0** — see [LICENSE](LICENSE).

---

<div align="center">

## Maintainer

**Rahul S** · _Engineering-Focused Builder_

Civil engineering student who codes. I build AI apps, browser tools, games, and construction software — 40+ projects live. I don't just build websites, I engineer solutions.

[![Portfolio](https://img.shields.io/badge/Portfolio-rahulshyam--portfolio.vercel.app-000000?style=flat&logo=vercel)](https://rahulshyam-portfolio.vercel.app/)
[![GitHub](https://img.shields.io/badge/GitHub-@rahulcvwebsitehosting-181717?style=flat&logo=github)](https://github.com/rahulcvwebsitehosting)
[![LinkedIn](https://img.shields.io/badge/LinkedIn-rahulshyamcivil-0A66C2?style=flat&logo=linkedin)](https://in.linkedin.com/in/rahulshyamcivil)
[![X](https://img.shields.io/badge/X-@RahulShyamCV-000000?style=flat&logo=x)](https://x.com/RahulShyamCV)

</div>

_Bug-fix fork of the original Pluely project. Maintained independently — fixes, improvements, and issues handled here._

[Report a Bug](https://github.com/rahulcvwebsitehosting/pluely-fixes/issues) · [View Source](https://github.com/rahulcvwebsitehosting/pluely-fixes)

</div>
