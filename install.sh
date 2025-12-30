#!/bin/bash
set -e

# wtenv installer script
# Usage: curl -fsSL https://raw.githubusercontent.com/USERNAME/wtenv/main/install.sh | bash

VERSION="${WTENV_VERSION:-latest}"
REPO="USERNAME/wtenv"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Linux*)     OS="linux" ;;
        Darwin*)    OS="macos" ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)   ARCH="x64" ;;
        aarch64|arm64)  ARCH="arm64" ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            ;;
    esac
}

# Get download URL
get_download_url() {
    local binary_name="wtenv-${OS}-${ARCH}"

    if [ "$OS" = "windows" ]; then
        binary_name="${binary_name}.exe"
    fi

    if [ "$VERSION" = "latest" ]; then
        DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${binary_name}"
    else
        DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${binary_name}"
    fi
}

# Determine install directory
get_install_dir() {
    if [ -n "$WTENV_INSTALL_DIR" ]; then
        INSTALL_DIR="$WTENV_INSTALL_DIR"
    elif [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="${HOME}/.local/bin"
    fi

    mkdir -p "$INSTALL_DIR"
}

# Download and install
install() {
    local tmp_file
    tmp_file=$(mktemp)

    info "Downloading wtenv from ${DOWNLOAD_URL}..."

    if command -v curl &> /dev/null; then
        curl -fsSL "$DOWNLOAD_URL" -o "$tmp_file"
    elif command -v wget &> /dev/null; then
        wget -q "$DOWNLOAD_URL" -O "$tmp_file"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    local binary_name="wtenv"
    if [ "$OS" = "windows" ]; then
        binary_name="wtenv.exe"
    fi

    mv "$tmp_file" "${INSTALL_DIR}/${binary_name}"
    chmod +x "${INSTALL_DIR}/${binary_name}"

    info "wtenv installed to ${INSTALL_DIR}/${binary_name}"
}

# Check if install directory is in PATH
check_path() {
    if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
        warn "${INSTALL_DIR} is not in your PATH"
        echo ""
        echo "Add the following to your shell configuration file:"
        echo ""
        echo "  export PATH=\"\$PATH:${INSTALL_DIR}\""
        echo ""
    fi
}

# Verify installation
verify() {
    if command -v wtenv &> /dev/null; then
        info "Installation successful!"
        wtenv --version
    else
        warn "wtenv was installed but may not be in your PATH yet."
        echo "Run: ${INSTALL_DIR}/wtenv --version"
    fi
}

main() {
    echo "Installing wtenv..."
    echo ""

    detect_os
    detect_arch
    get_download_url
    get_install_dir
    install
    check_path

    echo ""
    verify
}

main
