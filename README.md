---
Notice: This repository is archived and not actively maintained. See .github/adr/0002-archive-fodorian-legacy.md for rationale and details.
---

# Fodorian Legacy

An AI copilot for the terminal — chat with an LLM that can execute commands inside sandboxed containers, capture screenshots with OCR, and help you debug stuff.

I built this in 6 weeks (March 21 – April 10, 2025) after only a month of coding. Before this, I had never touched Rust, Tauri, or any of this. The code is rough. There are no tests. Some variabl[...]

## What it does

- **Chat with Google Vertex AI** — send a prompt, get a response. The AI acts as a technical architect and proposes solutions.
- **Execute commands in a sandbox** — the AI can suggest shell commands. If you approve them, they run inside an ephemeral Podman container with no network, read-only filesystem, and tight resou[...]
- **Screenshot + OCR** — select a region of your screen (grim/slurp for Wayland, flameshot or maim for X11), optionally run OCR with tesseract, and the AI analyzes it.
- **Attach files** — send log files, code, configs for the AI to read.
- **Multiple sessions** — switch between independent chats.

## How it works (roughly)

```
Frontend (React) → Tauri IPC → Rust backend → GCP Vertex AI
                                              → Podman sandbox
```

The Rust side has three modules:

| File | Does |
|------|------|
| `gcp.rs` | Loads env vars, handles GCP auth with a 50-min token cache, calls the Vertex AI Reasoning Engine |
| `sandbox.rs` | Validates commands against a whitelist, runs them in an Alpine container with `--network none`, `--cap-drop ALL`, 256 MB RAM, 0.5 CPU |
| `capture.rs` | Screenshot capture via grim/flameshot/maim + optional OCR |

## Why it looks like this

I started learning to code on February 28, 2025. No prior experience — I learned HTML, CSS, JavaScript, then jumped into Rust and Tauri because I wanted to build something real, not a todo app.

You'll notice:
- Spanish/English mix in function names and error messages. I was reading docs in English but thinking in Spanish.
- No tests. I didn't know how to write them yet.
- `debug!` macros everywhere. I was debugging by logging everything to stderr. They're gated to dev builds now.
- Some patterns are naive (string-based command validation, `any` types that I later replaced with proper interfaces).

This is not polished production code. It's a first project by someone who barely knew what a struct was 6 weeks earlier.

## Quick start

You'll need Rust, Node.js, Podman, and a Google Cloud account with Vertex AI set up.

```bash
git clone git@github.com:nicolasjorquera-cloud/fodorian-legacy.git
cd fodorian-legacy
npm install

# Set up GCP credentials
cp .env.example .env
# Edit .env with your project ID, location, and reasoning engine ID

npm run tauri dev
```

## Env vars

Put these in `.env` (see `.env.example`):

| Variable | What |
|----------|------|
| `GOOGLE_PROJECT_ID` | Your GCP project |
| `GOOGLE_LOCATION` | Region like `us-central1` |
| `GOOGLE_ENGINE_ID` | Vertex AI Reasoning Engine ID |

Or drop a `.env` at `~/Documents/gcp-c/.env` and the app picks it up.

## Security notes

No credentials are hardcoded. Everything comes from env vars. The Podman sandbox drops all capabilities, blocks network access, limits PIDs and memory. Commands go through a whitelist validator be[...]

## License

MIT
