# youtube-downloader

TUI em Rust para baixar **vídeos** e **áudios** do YouTube, com boa UI/UX no terminal.

Backend de download: **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** (subprocesso).  
Interface: **[Ratatui](https://ratatui.rs/)** + Crossterm + Tokio.

## Dependências do sistema

| Ferramenta | Obrigatório | Função |
|------------|-------------|--------|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | **Sim** | Extrair e baixar mídia do YouTube |
| [ffmpeg](https://ffmpeg.org/) | Recomendado | Merge vídeo+áudio e conversão de áudio (mp3/m4a/opus) |

### Instalação rápida

```bash
# Arch
sudo pacman -S yt-dlp ffmpeg

# Debian/Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg
```

## Build e execução

```bash
cargo run --release
```

Binário: `target/release/youtube-downloader`.

## Uso

1. Cole a URL do YouTube no campo principal.
2. Escolha **Vídeo** ou **Áudio** (`v` / `a`) e a qualidade (`1`–`5`).
3. `Enter` busca os metadados (título, canal, duração).
4. No preview, `Enter` adiciona à fila e inicia o download.
5. Acompanhe progresso em **Fila** (`f`).

### Atalhos principais

| Tecla | Ação |
|-------|------|
| `Enter` | Buscar / confirmar download |
| `v` / `a` | Modo vídeo / áudio |
| `1`–`5` | Qualidade (Melhor → Pior) |
| `f` | Fila |
| `s` | Configurações |
| `p` | Cancelar download ativo |
| `o` | Abrir pasta de saída |
| `?` | Ajuda |
| `q` | Sair |

## Configuração

Arquivo: `~/.config/youtube-downloader/config.toml`

```toml
output_dir = "/home/voce/Downloads/youtube-downloader"
output_template = "%(title)s [%(id)s].%(ext)s"
default_mode = "video"
default_quality = "best"
default_audio_format = "m4a"
```

Padrão de saída: `~/Downloads/youtube-downloader/` (ou home se Downloads não existir).

## Stack

- **ratatui** + **crossterm** — TUI
- **tokio** — runtime async e subprocessos
- **tui-input** — campo de texto
- **serde / toml / serde_json** — config e metadata yt-dlp
- **yt-dlp** + **ffmpeg** — download real (externos)

## Licença

MIT
