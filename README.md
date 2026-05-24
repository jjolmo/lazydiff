# LazyDiff

Human-readable PR diff viewer powered by AI. Stop reading raw diffs — let Claude explain what actually changed.

![LazyDiff](https://cidwel.com/mini/lazydiff/screenshot.png)

## What it does

- Paste a **GitHub PR/branch URL** or open a **local repo**
- See the file tree with additions/deletions
- Click any file to get an **AI-generated summary** of what changed, in plain language
- Toggle between **Semantic view** (human-readable) and **Code view** (raw diff)
- Switch between **Human** and **Caveman** summary styles
- Flow diagram shows what each file calls and what calls it

## Install

### macOS

```bash
curl -sL https://raw.githubusercontent.com/jjolmo/lazydiff/main/update_mac.sh | bash
```

Or download the `.dmg` from [Releases](https://github.com/jjolmo/lazydiff/releases/latest).

If macOS blocks it: `xattr -cr /Applications/LazyDiff.app`

### Windows

Download the `.exe` installer from [Releases](https://github.com/jjolmo/lazydiff/releases/latest).

### Linux

Download from [Releases](https://github.com/jjolmo/lazydiff/releases/latest):
- `.AppImage` — portable, run anywhere
- `.deb` — Debian/Ubuntu: `sudo dpkg -i LazyDiff_*.deb`
- `.rpm` — Fedora/RHEL: `sudo rpm -i LazyDiff-*.rpm`

## Setup

1. Open the app
2. Click ⚙️ → **Claude API** tab
3. Paste your [Anthropic API key](https://console.anthropic.com/)
4. Done — click any file to get AI summaries

## Usage

### GitHub mode
- Enter `owner/repo` (e.g. `facebook/react`)
- Select head and base branches from the typeahead
- Click **Load Repo**

### Local mode
- Click **Select repo...** to pick a local git directory
- Current branch is auto-detected as head
- Base defaults to `trunk`, `main`, or `master`
- Click **Load Repo**

### Summary styles
- **Human** — full sentences, descriptive
- **Caveman** — compressed style: 2-5 words, no grammar, just facts

## Stack

- **Tauri v2** — native desktop shell
- **SvelteKit 5** — reactive UI framework
- **Rust** — backend (GitHub API, git commands, Claude API)
- **Tailwind v4** — styling
- **GitHub Actions** — CI builds for all platforms on every tag

## License

MIT

## Author

[Javier Olmo](https://github.com/jjolmo)
