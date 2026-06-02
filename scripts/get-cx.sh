#!/bin/sh
# Installer script for cx (conda-express)
#
# Usage:
#   curl -fsSL https://jezdez.github.io/conda-express/get-cx.sh | sh
#   wget -qO- https://jezdez.github.io/conda-express/get-cx.sh | sh
#
# Options (via environment variables):
#   CX_INSTALL_DIR       — where to place the binary (default: ~/.local/bin)
#   CX_VERSION           — version to install, without "v" prefix (default: latest)
#   CX_NO_PATH_UPDATE    — set to non-empty to skip shell profile modification
#   CX_NO_BOOTSTRAP      — set to non-empty to skip running `cx bootstrap`
#   CX_SKIP_VERIFY       — set to non-empty to skip checksum verification

# shellcheck disable=SC2218
set -eu

__wrap__() {

REPO="jezdez/conda-express"
BINARY_NAME="cx"

detect_os() {
    _os="$(uname -s)"
    case "$_os" in
        Linux)  echo "linux" ;;
        Darwin) echo "macos" ;;
        *)      err "Unsupported operating system: %s" "$_os" ;;
    esac
}

detect_arch() {
    _arch="$(uname -m)"
    case "$_arch" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              err "Unsupported architecture: %s" "$_arch" ;;
    esac
}

map_target() {
    _os="$1"
    _arch="$2"
    case "${_os}-${_arch}" in
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
        macos-x86_64)   echo "x86_64-apple-darwin" ;;
        macos-aarch64)  echo "aarch64-apple-darwin" ;;
        *)              err "No prebuilt binary for %s %s" "$_os" "$_arch" ;;
    esac
}

download() {
    _url="$1"
    _dest="$2"

    if [ ! -t 1 ]; then
        CURL_OPTS="--silent"
        WGET_OPTS="--no-verbose"
    else
        CURL_OPTS=""
        WGET_OPTS="--show-progress"
    fi

    if check_cmd curl; then
        _http_code="$(curl -fSL $CURL_OPTS "$_url" --output "$_dest" --write-out "%{http_code}")" || {
            err "Download failed. Is curl working? URL: %s" "$_url"
        }
        if [ "$_http_code" -lt 200 ] || [ "$_http_code" -gt 299 ]; then
            err "Download failed with HTTP %s: %s" "$_http_code" "$_url"
        fi
    elif check_cmd wget; then
        wget $WGET_OPTS --output-document="$_dest" "$_url" || {
            err "Download failed. Is wget working? URL: %s" "$_url"
        }
    else
        err "Need curl or wget to download files"
    fi
}

verify_checksum() {
    _url="$1"
    _file="$2"

    if [ -n "${CX_SKIP_VERIFY:-}" ]; then
        warn "Skipping checksum verification because CX_SKIP_VERIFY is set"
        return 0
    fi

    _tmp_sha="$(mktemp "${TMPDIR:-/tmp}/.cx_sha.XXXXXXXX")"
    if ! download "${_url}.sha256" "$_tmp_sha" 2>/dev/null; then
        rm -f "$_tmp_sha"
        err "Checksum file not available: %s.sha256" "$_url"
    fi

    _expected="$(awk '{print $1}' "$_tmp_sha")"
    rm -f "$_tmp_sha"

    if check_cmd sha256sum; then
        _actual="$(sha256sum "$_file" | awk '{print $1}')"
    elif check_cmd shasum; then
        _actual="$(shasum -a 256 "$_file" | awk '{print $1}')"
    else
        err "No sha256sum or shasum found; install one or set CX_SKIP_VERIFY=1"
    fi

    if [ "$_expected" != "$_actual" ]; then
        err "Checksum mismatch!\n  expected: %s\n  actual:   %s" "$_expected" "$_actual"
    fi

    info "Checksum OK"
}

update_shell_profile() {
    _dir="$1"

    if echo "$PATH" | tr ':' '\n' | grep -qx "$_dir" 2>/dev/null; then
        return 0
    fi

    _quoted_dir="$(quote_profile_path "$_dir")"
    _line="export PATH=${_quoted_dir}:\$PATH"

    case "$(basename "${SHELL:-}")" in
        bash)
            append_line_if_missing "$HOME/.bashrc" "$_line"
            ;;
        zsh)
            append_line_if_missing "$HOME/.zshrc" "$_line"
            ;;
        fish)
            _line="set -gx PATH ${_quoted_dir} \$PATH"
            append_line_if_missing "$HOME/.config/fish/config.fish" "$_line"
            ;;
        *)
            warn "%s is not in your PATH." "$_dir"
            warn "Add it with:  %s" "$_line"
            return 0
            ;;
    esac
}

quote_profile_path() {
    printf "%s" "$1" | sed "s/'/'\\\\''/g; 1s/^/'/; \$s/\$/'/"
}

append_line_if_missing() {
    _file="$1"
    _line="$2"

    if [ -f "$_file" ] && grep -Fxq "$_line" "$_file" 2>/dev/null; then
        return 0
    fi

    if [ ! -f "$_file" ]; then
        _parent="${_file%/*}"
        if [ "$_parent" != "$_file" ]; then
            mkdir -p "$_parent"
        fi
        touch "$_file"
    fi

    printf '\n%s\n' "$_line" >> "$_file"
    info "Updated %s — restart your shell or run:  source %s" "$_file" "$_file"
}

warn_legacy_prefix() {
    if [ -d "$HOME/.cx" ]; then
        warn "Found legacy cx prefix at %s/.cx" "$HOME"
        warn "Current cx bootstraps into %s/.conda/express." "$HOME"
        warn "Keep %s/.cx until you have moved any environments you still need." "$HOME"
        warn "Upgrade guide: https://jezdez.github.io/conda-express/guides/upgrade-from-early-cx/"
    fi
}

check_cmd() {
    command -v "$1" >/dev/null 2>&1
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "Required command not found: %s" "$1"
    fi
}

info() {
    _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;32m>\033[0m $_fmt\n" "$@"
}

warn() {
    _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;33m!\033[0m $_fmt\n" "$@" >&2
}

err() {
    _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;31mx\033[0m $_fmt\n" "$@" >&2
    exit 1
}

main() {
    need_cmd uname
    need_cmd chmod
    need_cmd mkdir

    _os="$(detect_os)"
    _arch="$(detect_arch)"
    _target="$(map_target "$_os" "$_arch")"
    _version="${CX_VERSION:-latest}"
    _install_dir="${CX_INSTALL_DIR:-$HOME/.local/bin}"

    if [ "$_version" = "latest" ]; then
        _url="https://github.com/${REPO}/releases/latest/download/cx-${_target}"
    else
        _url="https://github.com/${REPO}/releases/download/${_version#v}/cx-${_target}"
    fi

    _tmp="$(mktemp "${TMPDIR:-/tmp}/.cx_install.XXXXXXXX")"
    trap 'rm -f "$_tmp"' EXIT

    printf "\n"
    info "Installing cx (conda-express) for %s %s" "$_os" "$_arch"
    info "Downloading %s" "$_url"

    download "$_url" "$_tmp"

    if [ ! -s "$_tmp" ]; then
        err "Downloaded file is empty. Check the URL or try again."
    fi

    info "Verifying checksum"
    verify_checksum "$_url" "$_tmp"

    chmod +x "$_tmp"
    mkdir -p "$_install_dir"
    mv -f "$_tmp" "${_install_dir}/${BINARY_NAME}"
    trap - EXIT

    info "Installed cx to %s/%s" "$_install_dir" "$BINARY_NAME"

    if [ -z "${CX_NO_PATH_UPDATE:-}" ]; then
        update_shell_profile "$_install_dir"
    fi

    warn_legacy_prefix

    if [ -z "${CX_NO_BOOTSTRAP:-}" ]; then
        printf "\n"
        info "Running cx bootstrap..."
        "${_install_dir}/${BINARY_NAME}" bootstrap
    fi

    printf "\n"
    info "Done! Run 'cx --help' to get started."
    printf "\n"
}

main "$@"
} && __wrap__ "$@"
