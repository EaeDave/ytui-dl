# ytui-dl

**YouTube no terminal.** TUI em Rust para baixar vídeos e áudios do YouTube.

Backend: **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** · Interface: **[Ratatui](https://ratatui.rs/)**

> 🇬🇧 English docs: [README.md](./README.md)

## Instalação rápida

### 1. Dependências de runtime

| Ferramenta | Obrigatório | Função |
|------------|-------------|--------|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | **Sim** | Extrair e baixar mídia |
| [ffmpeg](https://ffmpeg.org/) | Recomendado | Merge de streams / converter áudio |

```bash
# Arch
sudo pacman -S yt-dlp ffmpeg

# Debian/Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg

# Windows (winget)
winget install yt-dlp.yt-dlp Gyan.FFmpeg
# ou: scoop install yt-dlp ffmpeg
```

### 2. Instalar o binário

#### Linux

```bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
# → ~/.local/bin/ytui-dl
```

#### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.ps1 | iex
# → %LOCALAPPDATA%\ytui-dl\bin\ytui-dl.exe (+ PATH do usuário)
```

Use o [Windows Terminal](https://aka.ms/terminal) para a melhor experiência de TUI.

#### Depois de instalar

```bash
ytui-dl
ytui-dl --version
```

#### Atualizar / desinstalar

```bash
ytui-dl --update
ytui-dl --update --force
ytui-dl --uninstall
```

### Build a partir do código

```bash
cargo install --git https://github.com/EaeDave/ytui-dl
```

## Uso

1. Cole a URL do YouTube na tela inicial.
2. Escolha **Vídeo** ou **Áudio** (`v` / `a`), o **perfil** e a qualidade (`1`–`5`).
3. `Enter` busca metadados (título, canal, duração).
4. No preview, `Enter` enfileira e inicia o download.
5. Acompanhe em **Fila** (`f`).

### Perfis de saída

| Perfil | Quando usar | Notas |
|--------|-------------|--------|
| **Melhor qualidade** (padrão) | Guardar / assistir | Pode ser VP9/AV1 + Opus |
| **Compatível · WhatsApp** | Enviar no Zap / apps chatos | Prefere H.264 + AAC; pode reencode via ffmpeg (tecla `w`) |

Perfil WhatsApp exige **ffmpeg**.

### Idioma

A UI começa em **inglês**. Troque para **Português (BR)** em **Configurações** (`s`) → Idioma (`←`/`→` ou `Enter`) e salve.

Também em `~/.config/ytui-dl/config.toml`:

```toml
language = "pt-BR"    # ou "en"
```

### Atalhos

| Tecla | Ação |
|-------|------|
| `Enter` | Buscar / confirmar download |
| `v` / `a` | Modo vídeo / áudio |
| `w` / `b` | Perfil WhatsApp / Melhor qualidade |
| `←` / `→` | Alternar qualidade, áudio ou perfil (com foco) |
| `1`–`5` | Presets de qualidade (Melhor → Pior) |
| `f` | Fila |
| `s` | Configurações |
| `p` | Cancelar download ativo |
| `o` | Abrir pasta de saída |
| `u` | Info de atualização (se houver release nova) |
| `?` | Ajuda |
| `q` | Sair |

Na abertura (Linux), o ytui-dl consulta as GitHub Releases em background. Se houver versão mais nova, um badge amarelo aparece no header. Pressione **`u`** → confirme com **Enter** → a instalação roda em background (SHA256 + replace atômico) → **R** / **Enter** reinicia na build nova. Também existe **`ytui-dl --update`** no shell.

## Configuração

Arquivo: `~/.config/ytui-dl/config.toml`

```toml
output_dir = "/home/voce/Downloads/ytui-dl"
output_template = "%(title)s [%(id)s].%(ext)s"
default_mode = "video"
default_profile = "best"   # ou "whats_app" / "whatsapp"
default_quality = "best"
default_audio_format = "m4a"
language = "pt-BR"
auto_open = true   # abre o arquivo no player do sistema ao terminar
```

Com `auto_open` ligado (padrão), o vídeo/áudio concluído abre no app padrão do SO (`xdg-open` no Linux). Alternar em **Configurações** ou `auto_open = false` no config.

Padrão de saída: `~/Downloads/ytui-dl/` (ou home se Downloads não existir).

## Releases

Binários oficiais: [GitHub Releases](https://github.com/EaeDave/ytui-dl/releases)

## Stack

- **ratatui** + **crossterm** — TUI
- **tokio** — runtime async e subprocessos
- **tui-input** — campo de texto
- **serde / toml / serde_json** — config e metadata yt-dlp
- **yt-dlp** + **ffmpeg** — download real (externos)

## Licença

MIT
