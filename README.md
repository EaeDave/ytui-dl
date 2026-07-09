# ytui-dl

**YouTube in the terminal.** A Rust TUI for downloading YouTube videos and audio.

Backend: **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** · UI: **[Ratatui](https://ratatui.rs/)**

> 🇧🇷 Documentação em português: [README.pt-BR.md](./README.pt-BR.md)

## Quick install

### 1. Runtime dependencies

| Tool | Required | Role |
|------|----------|------|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | **Yes** | Extract and download media |
| [ffmpeg](https://ffmpeg.org/) | Recommended | Merge streams / convert audio |

```bash
# Arch
sudo pacman -S yt-dlp ffmpeg

# Debian/Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg
```

### 2. Install the binary (Linux)

Installs to **`~/.local/bin/ytui-dl`** and upgrades when a newer release exists:

```bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
```

Then:

```bash
ytui-dl
```

#### Update / force / uninstall

```bash
# Update if a newer release is available (default)
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash

# Reinstall even on the same version
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --force

# Installed vs remote
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --check

# Remove binary
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --uninstall

# System-wide (may prompt for sudo)
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --system
```

If `~/.local/bin` is not on your `PATH`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc   # or ~/.zshrc
```

### Build from source

```bash
cargo install --git https://github.com/EaeDave/ytui-dl
# or
git clone https://github.com/EaeDave/ytui-dl
cd ytui-dl && cargo install --path .
```

## Usage

1. Paste a YouTube URL on the home screen.
2. Choose **Video** or **Audio** (`v` / `a`) and quality (`1`–`5`).
3. `Enter` fetches metadata (title, channel, duration).
4. On preview, `Enter` enqueues and starts the download.
5. Track progress in **Queue** (`f`).

### Language

UI defaults to **English**. Change to **Português (BR)** in **Settings** (`s`) → Language (`←`/`→` or `Enter`), then save.

Also stored in `~/.config/ytui-dl/config.toml`:

```toml
language = "en"    # or "pt-BR"
```

### Shortcuts

| Key | Action |
|-----|--------|
| `Enter` | Fetch / confirm download |
| `v` / `a` | Video / audio mode |
| `←` / `→` | Cycle quality or audio format |
| `1`–`5` | Quality presets (Best → Worst) |
| `f` | Queue |
| `s` | Settings |
| `p` | Cancel active download |
| `o` | Open output folder |
| `?` | Help |
| `q` | Quit |

## Configuration

File: `~/.config/ytui-dl/config.toml`

```toml
output_dir = "/home/you/Downloads/ytui-dl"
output_template = "%(title)s [%(id)s].%(ext)s"
default_mode = "video"
default_quality = "best"
default_audio_format = "m4a"
language = "en"
```

Default output: `~/Downloads/ytui-dl/` (or home if Downloads is missing).

## Releases

Official binaries: [GitHub Releases](https://github.com/EaeDave/ytui-dl/releases)

Example assets:

- `ytui-dl-x86_64-unknown-linux-gnu`
- `*.sha256`

## Stack

- **ratatui** + **crossterm** — TUI
- **tokio** — async runtime and subprocesses
- **tui-input** — text input
- **serde / toml / serde_json** — config and yt-dlp metadata
- **yt-dlp** + **ffmpeg** — actual download (external)

## License

MIT
