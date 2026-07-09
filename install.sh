#!/usr/bin/env bash
# ytui-dl installer — install / update / uninstall
#
#   curl -fsSL https://raw.githubusercontent.com/EaeDave/ytui-dl/main/install.sh | bash
#   curl -fsSL ... | bash -s -- --uninstall
#   curl -fsSL ... | bash -s -- --force
set -euo pipefail

readonly REPO="EaeDave/ytui-dl"
readonly BIN_NAME="ytui-dl"
readonly GITHUB_API="https://api.github.com/repos/${REPO}"
readonly GITHUB_RELEASES="https://github.com/${REPO}/releases"

PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${BIN_DIR:-}"
FORCE=0
UNINSTALL=0
SYSTEM=0
CHECK_ONLY=0

info()  { printf '==> %s\n' "$*"; }
warn()  { printf '!!  %s\n' "$*" >&2; }
die()   { printf 'error: %s\n' "$*" >&2; exit 1; }

usage() {
  cat <<EOF
ytui-dl installer

Usage:
  install.sh [options]

Options:
  --prefix DIR    Install prefix (default: ~/.local → binary in DIR/bin)
  --bin-dir DIR   Exact directory for the binary (overrides --prefix)
  --system        Install to /usr/local/bin (may require sudo)
  --force         Reinstall even if the same version is already installed
  --uninstall     Remove ytui-dl from the install location
  --check         Show installed vs latest remote version
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
  printf '%s/%s\n' "$(resolve_bin_dir)" "${BIN_NAME}"
}

installed_version() {
  local path
  path="$(installed_path)"
  if [[ -x "$path" ]]; then
    "$path" --version 2>/dev/null | head -n1 | awk '{print $NF}' | sed 's/^v//' || true
    return
  fi
  if command -v "$BIN_NAME" >/dev/null 2>&1; then
    "$BIN_NAME" --version 2>/dev/null | head -n1 | awk '{print $NF}' | sed 's/^v//' || true
  fi
}

# Prints: TAG\tASSET_URL\tASSET_NAME
fetch_latest_release() {
  need_cmd curl
  local json tag asset_name asset_url target
  target="$(detect_target)"
  asset_name="${BIN_NAME}-${target}"

  json="$(curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "X-GitHub-Api-Version: 2022-11-28" \
    "${GITHUB_API}/releases/latest")" \
    || die "não foi possível consultar releases (existe release pública em ${GITHUB_RELEASES}?)"

  if command -v jq >/dev/null 2>&1; then
    tag="$(printf '%s' "$json" | jq -r '.tag_name // empty')"
    asset_url="$(printf '%s' "$json" | jq -r --arg n "$asset_name" \
      '.assets[]? | select(.name==$n) | .browser_download_url' | head -n1)"
  else
    tag="$(printf '%s' "$json" \
      | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' \
      | head -n1 \
      | sed 's/.*"\([^"]*\)"$/\1/')"
    asset_url="$(printf '%s' "$json" \
      | grep -oE "https://[^\"]+/${asset_name}\"" \
      | head -n1 \
      | tr -d '"')"
  fi

  [[ -n "$tag" ]] || die "nenhuma release encontrada em ${REPO}"
  [[ -n "$asset_url" ]] || die "asset '${asset_name}' não encontrado na release ${tag}. Veja ${GITHUB_RELEASES}"

  printf '%s\t%s\t%s\n' "$tag" "$asset_url" "$asset_name"
}

ensure_dir() {
  local dir="$1"
  [[ -d "$dir" ]] && return 0
  info "criando ${dir}"
  if mkdir -p "$dir" 2>/dev/null; then
    return 0
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo mkdir -p "$dir"
  else
    die "sem permissão para criar ${dir}"
  fi
}

install_file() {
  local src="$1" dest="$2"
  if install -m 755 "$src" "$dest" 2>/dev/null; then
    return 0
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo install -m 755 "$src" "$dest"
  else
    die "sem permissão para instalar em ${dest}"
  fi
}

remove_file() {
  local path="$1"
  if rm -f "$path" 2>/dev/null; then
    return 0
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo rm -f "$path"
  else
    die "sem permissão para remover ${path}"
  fi
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
  local path
  path="$(installed_path)"
  if [[ ! -e "$path" ]]; then
    if command -v "$BIN_NAME" >/dev/null 2>&1; then
      path="$(command -v "$BIN_NAME")"
    else
      die "${BIN_NAME} não está instalado em $(installed_path)"
    fi
  fi
  info "removendo ${path}"
  remove_file "$path"
  info "desinstalado."
  info "config (~/.config/ytui-dl) e downloads não foram removidos."
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

  # Optional checksum (uploaded as asset.sha256)
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
  check_path

  local final_ver
  final_ver="$("$dest" --version 2>/dev/null | awk '{print $NF}' || echo "$remote_ver")"
  info "pronto: ${dest} (${final_ver})"
  info "runtime: instale yt-dlp (obrigatório) e ffmpeg (recomendado)"
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
    --system)
      SYSTEM=1
      shift
      ;;
    --force)
      FORCE=1
      shift
      ;;
    --uninstall)
      UNINSTALL=1
      shift
      ;;
    --check)
      CHECK_ONLY=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "opção desconhecida: $1 (use --help)"
      ;;
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
