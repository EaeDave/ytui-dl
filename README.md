# ytui-dl

**YouTube no terminal.** TUI em Rust para baixar vídeos e áudios do YouTube.

Backend: **[yt-dlp](https://github.com/yt-dlp/yt-dlp)** · Interface: **[Ratatui](https://ratatui.rs/)**

## Instalação rápida

### 1. Dependências de runtime

| Ferramenta | Obrigatório | Função |
|------------|-------------|--------|
| [yt-dlp](https://github.com/yt-dlp/yt-dlp) | **Sim** | Extrair e baixar mídia do YouTube |
| [ffmpeg](https://ffmpeg.org/) | Recomendado | Merge vídeo+áudio e conversão de áudio |

```bash
# Arch
sudo pacman -S yt-dlp ffmpeg

# Debian/Ubuntu
sudo apt install yt-dlp ffmpeg

# macOS
brew install yt-dlp ffmpeg
```

### 2. Instalar o binário (Linux)

O script instala em **`~/.local/bin/ytui-dl`** (no PATH do usuário) e **atualiza sozinho** se existir release mais nova:

```bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
```

Depois:

```bash
ytui-dl
```

#### Atualizar / forçar / desinstalar

```bash
# Atualiza se houver release mais recente (padrão)
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash

# Reinstala mesmo na mesma versão
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --force

# Ver instalado vs remote
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --check

# Remover do PATH de instalação
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --uninstall

# Instalação system-wide (pode pedir sudo)
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --system
```

Se `~/.local/bin` não estiver no PATH, o script avisa. No bash/zsh:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
# ou ~/.zshrc
```

### Build a partir do código

```bash
cargo install --git https://github.com/EaeDave/ytui-dl
# ou
git clone https://github.com/EaeDave/ytui-dl
cd ytui-dl && cargo install --path .
```

## Uso

1. Cole a URL do YouTube no campo principal.
2. Escolha **Vídeo** ou **Áudio** (`v` / `a`) e a qualidade (`1`–`5`).
3. `Enter` busca os metadados (título, canal, duração).
4. No preview, `Enter` adiciona à fila e inicia o download.
5. Acompanhe o progresso em **Fila** (`f`).

### Atalhos principais

| Tecla | Ação |
|-------|------|
| `Enter` | Buscar / confirmar download |
| `v` / `a` | Modo vídeo / áudio |
| `←` / `→` | Alternar qualidade ou formato de áudio |
| `1`–`5` | Qualidade (Melhor → Pior) |
| `f` | Fila |
| `s` | Configurações |
| `p` | Cancelar download ativo |
| `o` | Abrir pasta de saída |
| `?` | Ajuda |
| `q` | Sair |

## Configuração

Arquivo: `~/.config/ytui-dl/config.toml`

```toml
output_dir = "/home/voce/Downloads/ytui-dl"
output_template = "%(title)s [%(id)s].%(ext)s"
default_mode = "video"
default_quality = "best"
default_audio_format = "m4a"
```

Padrão de saída: `~/Downloads/ytui-dl/` (ou home se Downloads não existir).

## Releases

Binários oficiais: [GitHub Releases](https://github.com/EaeDave/ytui-dl/releases)

Assets por arquitetura, por exemplo:

- `ytui-dl-x86_64-unknown-linux-gnu`
- `ytui-dl-aarch64-unknown-linux-gnu`
- `*.sha256` (checksums)

## Stack

- **ratatui** + **crossterm** — TUI
- **tokio** — runtime async e subprocessos
- **tui-input** — campo de texto
- **serde / toml / serde_json** — config e metadata yt-dlp
- **yt-dlp** + **ffmpeg** — download real (externos)

## Licença

MIT
