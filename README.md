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
# Preferred after first install
ytui-dl --update
ytui-dl --update --force   # reinstall same version
ytui-dl --uninstall        # remove binary (keeps config & downloads)

# Or via install script
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --force
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --check
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
2. Choose **Video** or **Audio** (`v` / `a`), **profile**, and quality (`1`–`5`).
3. `Enter` fetches metadata (title, channel, duration).
4. On preview, `Enter` enqueues and starts the download.
5. Track progress in **Queue** (`f`).

### Output profiles

| Profile | Use when | Notes |
|---------|----------|--------|
| **Best quality** (default) | Archiving / watching | May use VP9/AV1 + Opus |
| **Compatible · WhatsApp** | Sending on WhatsApp / picky apps | Prefers H.264 + AAC; may re-encode via ffmpeg (`w` key) |

WhatsApp profile requires **ffmpeg**.

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
| `w` / `b` | WhatsApp / Best quality profile |
| `←` / `→` | Cycle quality, audio, or profile (when focused) |
| `1`–`5` | Quality presets (Best → Worst) |
| `f` | Queue |
| `s` | Settings |
| `p` | Cancel active download |
| `o` | Open output folder |
| `u` | Update info (if a newer release exists) |
| `?` | Help |
| `q` | Quit |

On startup (Linux), ytui-dl checks GitHub Releases in the background. If a newer version is available, a yellow badge appears in the header. Press **`u`** → confirm with **Enter** → install runs in the background (SHA256 + atomic replace) → **R** / **Enter** restarts into the new build. You can still use **`ytui-dl --update`** from the shell.

## Configuration

File: `~/.config/ytui-dl/config.toml`

```toml
output_dir = "/home/you/Downloads/ytui-dl"
output_template = "%(title)s [%(id)s].%(ext)s"
default_mode = "video"
default_profile = "best"   # or "whats_app" / "whatsapp"
default_quality = "best"
default_audio_format = "m4a"
language = "en"
auto_open = true   # open file with the system player when download finishes
```

When `auto_open` is on (default), the finished video/audio is opened with the OS default app (`xdg-open` on Linux). Toggle in **Settings** or set `auto_open = false` in the config.

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
