#!/usr/bin/env bash
# ytd installer — install / update / uninstall
#
#   curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
#   curl -fsSL ... | bash -s -- --uninstall
#   curl -fsSL ... | bash -s -- --force
set -euo pipefail

readonly REPO="EaeDave/ytui-dl"
readonly BIN_NAME="ytd"
readonly GITHUB_API="https://api.github.com/repos/${REPO}"
readonly GITHUB_RELEASES="https://github.com/${REPO}/releases"

PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${BIN_DIR:-}"
FORCE=0
UNINSTALL=0
SYSTEM=0
CHECK_ONLY=0
SKIP_DEPS=0

info()  { printf '==> %s\n' "$*"; }
warn()  { printf '!!  %s\n' "$*" >&2; }
die()   { printf 'error: %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<EOF
ytd installer (YouTube TUI downloader)

Usage:
  install.sh [options]

Options:
  --prefix DIR    Install prefix (default: ~/.local → binary in DIR/bin)
  --bin-dir DIR   Exact directory for the binary (overrides --prefix)
  --system        Install to /usr/local/bin (may require sudo)
  --force         Reinstall even if the same version is already installed
  --uninstall     Remove ytd from the install location
  --check         Show installed vs latest remote version
  --skip-deps     Do not offer to install yt-dlp / ffmpeg
  -h, --help      Show this help

Examples:
  curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
  curl -fsSL ... | bash -s -- --force
  curl -fsSL ... | bash -s -- --uninstall
  curl -fsSL ... | bash -s -- --system
EOF
}

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || die "comando obrigatório não encontrado: $1"
}

# True if $1 > $2 (semver-ish via sort -V)
version_gt() {
  local a="${1#v}" b="${2#v}"
  [[ -z "$b" ]] && return 0
  [[ -z "$a" ]] && return 1
  [[ "$a" == "$b" ]] && return 1
  [[ "$(printf '%s\n%s\n' "$a" "$b" | sort -V | tail -n1)" == "$a" ]]
}

version_eq() {
  local a="${1#v}" b="${2#v}"
  [[ "$a" == "$b" ]]
}

detect_target() {
  local os arch
  os="$(uname -s | tr '[:upper:]' '[:lower:]')"
  arch="$(uname -m)"

  case "$os" in
    linux) ;;
    darwin) die "macOS ainda não tem release binária; use: cargo install --git https://github.com/${REPO}" ;;
    *) die "SO não suportado: $os" ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    aarch64|arm64) arch="aarch64" ;;
    *) die "arquitetura não suportada: $arch" ;;
  esac

  echo "${arch}-unknown-linux-gnu"
}

resolve_bin_dir() {
  if [[ -n "${BIN_DIR}" ]]; then
    printf '%s\n' "${BIN_DIR}"
    return
  fi
  if [[ "$SYSTEM" -eq 1 ]]; then
    printf '%s\n' "/usr/local/bin"
    return
  fi
  printf '%s\n' "${PREFIX}/bin"
}

installed_path() {
  printf '%s/%s\n' "$(resolve_bin_dir)" "$BIN_NAME"
}

installed_version() {
  local path
  path="$(installed_path)"
  if [[ -x "$path" ]]; then
    "$path" --version 2>/dev/null | awk '{print $NF}'
    return 0
  fi
  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    "$BIN_NAME" --version 2>/dev/null | awk '{print $NF}'
    return 0
  fi
  return 1
}

ensure_dir() {
  local dir="$1"
  if [[ ! -d "$dir" ]]; then
    if [[ -w "$(dirname "$dir")" ]] || [[ "$dir" == "$HOME"* ]]; then
      mkdir -p "$dir"
    else
      info "criando ${dir} com sudo…"
      sudo mkdir -p "$dir"
    fi
  fi
}

install_file() {
  local src="$1" dest="$2"
  if [[ -w "$(dirname "$dest")" ]]; then
    install -m 755 "$src" "$dest"
  else
    info "instalando em ${dest} com sudo…"
    sudo install -m 755 "$src" "$dest"
  fi
}

remove_file() {
  local path="$1"
  if [[ -w "$(dirname "$path")" ]]; then
    rm -f "$path"
  else
    sudo rm -f "$path"
  fi
}

fetch_latest_release() {
  need_cmd curl
  local target tag asset_name asset_url api_json

  target="$(detect_target)"
  # Prefer new asset name; fall back to old ytui-dl-* for one transition if needed.
  asset_name="ytd-${target}"

  if api_json="$(curl -fsSL -A "ytd-install" \
      -H "Accept: application/vnd.github+json" \
      "${GITHUB_API}/releases/latest" 2>/dev/null)"; then
    tag="$(printf '%s' "$api_json" | sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"
  fi

  if [[ -z "${tag:-}" ]]; then
    local final
    final="$(curl -fsSLI -o /dev/null -w '%{url_effective}' -A "ytd-install" \
      "${GITHUB_RELEASES}/latest" 2>/dev/null || true)"
    tag="${final##*/}"
  fi
  [[ -n "${tag:-}" && "$tag" != "latest" ]] || die "não foi possível resolver a release mais recente"

  asset_url="${GITHUB_RELEASES}/download/${tag}/${asset_name}"
  if ! curl -fsSLI -A "ytd-install" "$asset_url" >/dev/null 2>&1; then
    # Transition: older tags used ytui-dl-* asset names
    asset_name="ytui-dl-${target}"
    asset_url="${GITHUB_RELEASES}/download/${tag}/${asset_name}"
  fi

  printf '%s\t%s\t%s\n' "$tag" "$asset_url" "$asset_name"
}

check_path() {
  local dir
  dir="$(resolve_bin_dir)"
  case ":${PATH}:" in
    *":${dir}:"*) ;;
    *)
      warn "${dir} não está no PATH"
      warn "adicione ao shell, por exemplo:"
      warn "  export PATH=\"${dir}:\$PATH\""
      warn "e reinicie o terminal (ou source no arquivo rc)."
      ;;
  esac
}

do_uninstall() {
  local path removed=0
  path="$(installed_path)"

  for candidate in "$path" \
      "$(dirname "$path")/ytui-dl" \
      "${HOME}/.local/bin/ytui-dl" \
      "/usr/local/bin/ytui-dl"; do
    if [[ -e "$candidate" ]]; then
      info "removendo ${candidate}"
      remove_file "$candidate"
      removed=1
    fi
  done

  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    local p
    p="$(command -v "$BIN_NAME")"
    if [[ -e "$p" ]]; then
      info "removendo ${p}"
      remove_file "$p" || true
      removed=1
    fi
  fi

  if [[ "$removed" -eq 0 ]]; then
    die "${BIN_NAME} não está instalado em $(installed_path)"
  fi
  info "desinstalado."
  info "config (~/.config/ytd) e downloads não foram removidos."
}

do_check() {
  local local_ver remote_tag remote_ver
  local_ver="$(installed_version || true)"
  remote_tag="$(fetch_latest_release | cut -f1)"
  remote_ver="${remote_tag#v}"
  printf 'instalado:  %s\n' "${local_ver:-"(não encontrado)"}"
  printf 'remote:     %s\n' "$remote_ver"
  if [[ -z "${local_ver}" ]]; then
    printf 'status:     não instalado\n'
  elif version_eq "$remote_ver" "$local_ver"; then
    printf 'status:     atualizado\n'
  elif version_gt "$remote_ver" "$local_ver"; then
    printf 'status:     atualização disponível (%s → %s)\n' "$local_ver" "$remote_ver"
  else
    printf 'status:     instalado é mais novo que a release (%s vs %s)\n' "$local_ver" "$remote_ver"
  fi
}

do_install() {
  need_cmd curl
  need_cmd install
  need_cmd uname
  need_cmd mktemp
  need_cmd sort
  need_cmd chmod

  local line tag asset_url asset_name remote_ver local_ver tmpdir bin_tmp dest
  line="$(fetch_latest_release)"
  tag="$(printf '%s' "$line" | cut -f1)"
  asset_url="$(printf '%s' "$line" | cut -f2)"
  asset_name="$(printf '%s' "$line" | cut -f3)"
  remote_ver="${tag#v}"
  local_ver="$(installed_version || true)"
  dest="$(installed_path)"

  info "release remota: ${tag}"
  if [[ -n "${local_ver}" ]]; then
    info "versão instalada: ${local_ver}"
    if version_eq "$remote_ver" "$local_ver" && [[ "$FORCE" -eq 0 ]]; then
      info "já está na versão mais recente (${local_ver}). Use --force para reinstalar."
      check_path
      if [[ "$SKIP_DEPS" -eq 0 ]]; then ensure_runtime_deps; fi
      exit 0
    fi
    if version_gt "$remote_ver" "$local_ver"; then
      info "atualizando ${local_ver} → ${remote_ver} (sobrescrevendo ${dest})"
    elif version_gt "$local_ver" "$remote_ver" && [[ "$FORCE" -eq 0 ]]; then
      warn "instalado (${local_ver}) é mais novo que a release (${remote_ver}). Use --force para sobrescrever."
      exit 0
    else
      info "reinstalando ${remote_ver} em ${dest}"
    fi
  else
    info "instalando ${remote_ver} em ${dest}"
  fi

  tmpdir="$(mktemp -d)"
  # shellcheck disable=SC2064
  trap "rm -rf '$tmpdir'" EXIT

  bin_tmp="${tmpdir}/${BIN_NAME}"
  info "baixando ${asset_name}"
  curl -fsSL "$asset_url" -o "$bin_tmp"
  chmod +x "$bin_tmp"

  local sum_url sum_file expected actual
  sum_url="${asset_url}.sha256"
  sum_file="${bin_tmp}.sha256"
  if curl -fsSL "$sum_url" -o "$sum_file" 2>/dev/null; then
    info "verificando checksum SHA256"
    need_cmd sha256sum
    expected="$(awk '{print $1}' "$sum_file")"
    actual="$(sha256sum "$bin_tmp" | awk '{print $1}')"
    [[ "$expected" == "$actual" ]] || die "checksum SHA256 não confere"
  fi

  ensure_dir "$(dirname "$dest")"
  install_file "$bin_tmp" "$dest"

  # Remove old command name if present
  local old
  for old in "$(dirname "$dest")/ytui-dl" "${HOME}/.local/bin/ytui-dl"; do
    if [[ -e "$old" && "$old" != "$dest" ]]; then
      info "removendo binário antigo: ${old}"
      remove_file "$old" || true
    fi
  done

  check_path

  local final_ver
  final_ver="$("$dest" --version 2>/dev/null | awk '{print $NF}' || echo "$remote_ver")"
  info "pronto: ${dest} (${final_ver})"
  info "rode:   ytd"

  if [[ "$SKIP_DEPS" -eq 0 ]]; then
    ensure_runtime_deps
  else
    info "runtime: yt-dlp (required) + ffmpeg (recommended)"
  fi
}

ensure_runtime_deps() {
  local need_yt=0 need_ff=0
  command -v yt-dlp >/dev/null 2>&1 || need_yt=1
  command -v ffmpeg >/dev/null 2>&1 || need_ff=1

  if [[ "$need_yt" -eq 0 && "$need_ff" -eq 0 ]]; then
    info "yt-dlp and ffmpeg already on PATH"
    return 0
  fi

  echo
  echo "Runtime dependencies:"
  if [[ "$need_yt" -eq 1 ]]; then echo "  - yt-dlp  (required)"; else echo "  - yt-dlp  (found)"; fi
  if [[ "$need_ff" -eq 1 ]]; then echo "  - ffmpeg  (recommended)"; else echo "  - ffmpeg  (found)"; fi
  echo

  if [[ ! -t 0 ]]; then
    warn "non-interactive shell — install deps yourself:"
    warn "  sudo pacman -S yt-dlp ffmpeg   # or apt / brew"
    return 0
  fi

  printf "Install missing deps with the system package manager? [Y/n] "
  read -r ans || ans=n
  if [[ "$ans" =~ ^[nN] ]]; then
    warn "skipped — ytd needs yt-dlp on PATH"
    return 0
  fi

  if command -v pacman >/dev/null 2>&1; then
    local pkgs=()
    [[ "$need_yt" -eq 1 ]] && pkgs+=(yt-dlp)
    [[ "$need_ff" -eq 1 ]] && pkgs+=(ffmpeg)
    info "sudo pacman -S ${pkgs[*]}"
    sudo pacman -S --needed "${pkgs[@]}"
  elif command -v apt-get >/dev/null 2>&1; then
    local pkgs=()
    [[ "$need_yt" -eq 1 ]] && pkgs+=(yt-dlp)
    [[ "$need_ff" -eq 1 ]] && pkgs+=(ffmpeg)
    info "sudo apt-get install -y ${pkgs[*]}"
    sudo apt-get update -qq
    sudo apt-get install -y "${pkgs[@]}"
  elif command -v brew >/dev/null 2>&1; then
    [[ "$need_yt" -eq 1 ]] && brew install yt-dlp
    [[ "$need_ff" -eq 1 ]] && brew install ffmpeg
  else
    warn "no supported package manager found (pacman/apt/brew)"
  fi
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      PREFIX="${2:?}"
      shift 2
      ;;
    --bin-dir)
      BIN_DIR="${2:?}"
      shift 2
      ;;
    --system) SYSTEM=1; shift ;;
    --force|-f) FORCE=1; shift ;;
    --uninstall) UNINSTALL=1; shift ;;
    --check) CHECK_ONLY=1; shift ;;
    --skip-deps) SKIP_DEPS=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) die "opção desconhecida: $1 (use --help)" ;;
  esac
done

if [[ "$UNINSTALL" -eq 1 ]]; then
  do_uninstall
  exit 0
fi

if [[ "$CHECK_ONLY" -eq 1 ]]; then
  do_check
  exit 0
fi

do_install
