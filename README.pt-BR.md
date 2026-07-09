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
```

### 2. Instalar o binário (Linux)

Instala em **`~/.local/bin/ytui-dl`** e atualiza sozinho se houver release mais nova:

```bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
```

Depois:

```bash
ytui-dl
```

#### Atualizar / forçar / desinstalar

```bash
# Preferido depois da primeira instalação
ytui-dl --update
ytui-dl --update --force   # reinstala a mesma versão

# Ou via script
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --force

# Instalado vs remote
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --check

# Remover binário
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --uninstall

# System-wide (pode pedir sudo)
curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash -s -- --system
```

Se `~/.local/bin` não estiver no `PATH`:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc   # ou ~/.zshrc
```

### Build a partir do código

```bash
cargo install --git https://github.com/EaeDave/ytui-dl
# ou
git clone https://github.com/EaeDave/ytui-dl
cd ytui-dl && cargo install --path .
```

## Uso

1. Cole a URL do YouTube na tela inicial.
2. Escolha **Vídeo** ou **Áudio** (`v` / `a`) e a qualidade (`1`–`5`).
3. `Enter` busca metadados (título, canal, duração).
4. No preview, `Enter` enfileira e inicia o download.
5. Acompanhe em **Fila** (`f`).

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
| `←` / `→` | Alternar qualidade ou formato de áudio |
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
default_quality = "best"
default_audio_format = "m4a"
language = "pt-BR"
```

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
