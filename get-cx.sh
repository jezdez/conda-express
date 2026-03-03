#!/bin/sh
# Installer script for cx (conda-express)
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/jezdez/conda-express/main/get-cx.sh | sh
#   wget -qO- https://raw.githubusercontent.com/jezdez/conda-express/main/get-cx.sh | sh
#
# Options (via environment variables):
#   CX_INSTALL_DIR       — where to place the binary (default: ~/.local/bin)
#   CX_VERSION           — version to install, without "v" prefix (default: latest)
#   CX_NO_PATH_UPDATE    — set to non-empty to skip shell profile modification
#   CX_NO_BOOTSTRAP      — set to non-empty to skip running `cx bootstrap`

set -eu

__wrap__() {

REPO="jezdez/conda-express"
BINARY_NAME="cx"

main() {
    need_cmd uname
    need_cmd chmod
    need_cmd mkdir

    local _os _arch _target _version _install_dir _url _tmp

    _os="$(detect_os)"
    _arch="$(detect_arch)"
    _target="$(map_target "$_os" "$_arch")"
    _version="${CX_VERSION:-latest}"
    _install_dir="${CX_INSTALL_DIR:-$HOME/.local/bin}"

    if [ "$_version" = "latest" ]; then
        _url="https://github.com/${REPO}/releases/latest/download/cx-${_target}"
    else
        _url="https://github.com/${REPO}/releases/download/v${_version#v}/cx-${_target}"
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

    if [ -z "${CX_NO_BOOTSTRAP:-}" ]; then
        printf "\n"
        info "Running cx bootstrap..."
        "${_install_dir}/${BINARY_NAME}" bootstrap
    fi

    printf "\n"
    info "Done! Run 'cx --help' to get started."
    printf "\n"
}

detect_os() {
    local _os
    _os="$(uname -s)"
    case "$_os" in
        Linux)  echo "linux" ;;
        Darwin) echo "macos" ;;
        *)      err "Unsupported operating system: %s" "$_os" ;;
    esac
}

detect_arch() {
    local _arch
    _arch="$(uname -m)"
    case "$_arch" in
        x86_64|amd64)   echo "x86_64" ;;
        aarch64|arm64)  echo "aarch64" ;;
        *)              err "Unsupported architecture: %s" "$_arch" ;;
    esac
}

map_target() {
    local _os="$1" _arch="$2"
    case "${_os}-${_arch}" in
        linux-x86_64)   echo "x86_64-unknown-linux-gnu" ;;
        linux-aarch64)  echo "aarch64-unknown-linux-gnu" ;;
        macos-x86_64)   echo "x86_64-apple-darwin" ;;
        macos-aarch64)  echo "aarch64-apple-darwin" ;;
        *)              err "No prebuilt binary for %s %s" "$_os" "$_arch" ;;
    esac
}

download() {
    local _url="$1" _dest="$2"

    if [ ! -t 1 ]; then
        CURL_OPTS="--silent"
        WGET_OPTS="--no-verbose"
    else
        CURL_OPTS=""
        WGET_OPTS="--show-progress"
    fi

    if check_cmd curl; then
        local _http_code
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
    local _url="$1" _file="$2" _expected _actual _tmp_sha

    _tmp_sha="$(mktemp "${TMPDIR:-/tmp}/.cx_sha.XXXXXXXX")"
    if ! download "${_url}.sha256" "$_tmp_sha" 2>/dev/null; then
        warn "Checksum file not available, skipping verification"
        rm -f "$_tmp_sha"
        return 0
    fi

    _expected="$(awk '{print $1}' "$_tmp_sha")"
    rm -f "$_tmp_sha"

    if check_cmd sha256sum; then
        _actual="$(sha256sum "$_file" | awk '{print $1}')"
    elif check_cmd shasum; then
        _actual="$(shasum -a 256 "$_file" | awk '{print $1}')"
    else
        warn "No sha256sum or shasum found, skipping verification"
        return 0
    fi

    if [ "$_expected" != "$_actual" ]; then
        err "Checksum mismatch!\n  expected: %s\n  actual:   %s" "$_expected" "$_actual"
    fi

    info "Checksum OK"
}

update_shell_profile() {
    local _dir="$1" _line

    if echo "$PATH" | tr ':' '\n' | grep -qx "$_dir" 2>/dev/null; then
        return 0
    fi

    _line="export PATH=\"${_dir}:\$PATH\""

    case "$(basename "${SHELL:-}")" in
        bash)
            append_line_if_missing "$HOME/.bashrc" "$_line"
            ;;
        zsh)
            append_line_if_missing "$HOME/.zshrc" "$_line"
            ;;
        fish)
            _line="set -gx PATH \"${_dir}\" \$PATH"
            append_line_if_missing "$HOME/.config/fish/config.fish" "$_line"
            ;;
        *)
            warn "%s is not in your PATH." "$_dir"
            warn "Add it with:  %s" "$_line"
            return 0
            ;;
    esac
}

append_line_if_missing() {
    local _file="$1" _line="$2"

    if [ -f "$_file" ] && grep -Fxq "$_line" "$_file" 2>/dev/null; then
        return 0
    fi

    [ -f "$_file" ] || touch "$_file"

    printf '\n%s\n' "$_line" >> "$_file"
    info "Updated %s — restart your shell or run:  source %s" "$_file" "$_file"
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
    local _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;32m>\033[0m $_fmt\n" "$@"
}

warn() {
    local _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;33m!\033[0m $_fmt\n" "$@" >&2
}

err() {
    local _fmt="$1"; shift
    # shellcheck disable=SC2059
    printf "  \033[1;31mx\033[0m $_fmt\n" "$@" >&2
    exit 1
}

main "$@"
} && __wrap__
