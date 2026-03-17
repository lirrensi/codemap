#!/bin/sh
# CodeMapper installer — Linux / macOS
# Usage: curl -fsSL https://raw.githubusercontent.com/lirrensi/codemap/main/scripts/install.sh | sh

set -eu

REPO="lirrensi/codemap"
BINARY="codemap"
INSTALL_DIR="${CODEMAP_INSTALL_DIR:-/usr/local/bin}"

# --- helpers ---
info()  { printf '\033[1;34m>\033[0m %s\n' "$1"; }
ok()    { printf '\033[1;32m✓\033[0m %s\n' "$1"; }
warn()  { printf '\033[1;33m!\033[0m %s\n' "$1"; }
die()   { printf '\033[1;31mx\033[0m %s\n' "$1" >&2; exit 1; }

need_cmd() {
    command -v "$1" >/dev/null 2>&1 || die "Required command '$1' not found"
}

# --- detect OS ---
detect_os() {
    os="$(uname -s)"
    case "$os" in
        Linux)  echo "linux" ;;
        Darwin) echo "macos" ;;
        *)      die "Unsupported OS: $os (Windows? Use install.ps1)" ;;
    esac
}

# --- detect arch ---
detect_arch() {
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  echo "x64" ;;
        aarch64|arm64) echo "arm64" ;;
        armv7l|armv6l) echo "arm" ;;
        *)             die "Unsupported architecture: $arch" ;;
    esac
}

# --- get latest version from GitHub ---
get_latest_version() {
    local url="https://api.github.com/repos/${REPO}/releases/latest"
    local version
    version=$(curl -fsSL "$url" | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    [ -n "$version" ] || die "Could not determine latest release version"
    echo "$version"
}

# --- main ---
main() {
    need_cmd curl
    need_cmd uname

    os=$(detect_os)
    arch=$(detect_arch)

    info "Detecting platform... ${os}-${arch}"

    # Get latest version
    info "Fetching latest release..."
    version=$(get_latest_version)
    info "Latest version: ${version}"

    # Build download URL
    filename="codemap-${version}-${os}-${arch}.zip"
    url="https://github.com/${REPO}/releases/download/${version}/${filename}"
    info "Downloading from: ${url}"

    # Create temp dir
    tmpdir=$(mktemp -d 2>/dev/null || mktemp -d -t codemap)
    trap 'rm -rf "$tmpdir"' EXIT

    archive="${tmpdir}/codemap.zip"
    curl -fSL --progress-bar -o "$archive" "$url" || {
        # Fallback: try without 'v' prefix in filename
        filename2="codemap-${version#v}-${os}-${arch}.zip"
        url2="https://github.com/${REPO}/releases/download/${version}/${filename2}"
        info "Retrying: ${url2}"
        curl -fSL --progress-bar -o "$archive" "$url2" || die "Download failed. Check https://github.com/${REPO}/releases"
    }

    # Extract
    info "Extracting..."
    need_cmd unzip
    unzip -qo "$archive" -d "$tmpdir"

    # Find the binary
    bin_path="${tmpdir}/${BINARY}"
    [ -f "$bin_path" ] || die "Binary '$BINARY' not found in archive"

    # Install
    info "Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "$bin_path" "${INSTALL_DIR}/${BINARY}"
    else
        sudo mv "$bin_path" "${INSTALL_DIR}/${BINARY}"
    fi

    chmod +x "${INSTALL_DIR}/${BINARY}"

    # Verify
    if command -v "$BINARY" >/dev/null 2>&1; then
        installed_version=$("$BINARY" --version 2>/dev/null || echo "unknown")
        ok "Installed ${BINARY} ${installed_version}"
    else
        warn "Installed to ${INSTALL_DIR}/${BINARY}"
        warn "Make sure ${INSTALL_DIR} is in your PATH"
        warn "  export PATH=\"${INSTALL_DIR}:\$PATH\""
    fi

    echo ""
    info "Quick start:"
    echo "  codemap                    # scan current directory"
    echo "  codemap setup              # add pre-commit hook"
    echo "  codemap --help             # see all options"
}

main
