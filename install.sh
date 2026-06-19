#!/usr/bin/env bash

# Exit immediately if a command exits with a non-zero status
set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo -e "${CYAN}=====================================================${NC}"
echo -e "${CYAN}       NeoTerminal Installer - Classy Rust Shell     ${NC}"
echo -e "${CYAN}=====================================================${NC}"

# Check for cargo/rust
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Rust/Cargo is not installed.${NC}"
    echo -e "Please install Rust first by running:"
    echo -e "  ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    exit 1
fi

echo -e "\n${BLUE}[1/4] Building release binary...${NC}"
cargo build --release

echo -e "\n${BLUE}[2/4] Installing binary to /usr/local/bin (requires sudo)...${NC}"
# Use sudo to copy the binary system-wide
sudo install -Dm755 target/release/neoterminal /usr/local/bin/neoterminal

# Clean up stale ~/.cargo/bin/neoterminal if it exists to avoid PATH conflicts
if [ -f "$HOME/.cargo/bin/neoterminal" ]; then
    echo -e "${YELLOW}Detected old neoterminal binary in ~/.cargo/bin/neoterminal. Removing it...${NC}"
    rm -f "$HOME/.cargo/bin/neoterminal"
fi

echo -e "\n${BLUE}[3/4] Creating desktop application shortcuts...${NC}"
DESKTOP_DIR="$HOME/Desktop"
LOCAL_APPS="$HOME/.local/share/applications"

# Ensure user applications directory exists
mkdir -p "$LOCAL_APPS"

# Create application menu launcher
cat <<EOF > "$LOCAL_APPS/neoterminal.desktop"
[Desktop Entry]
Version=1.0
Type=Application
Name=NeoTerminal
Comment=Classy Terminal Shell Wrapper in Rust
Exec=x-terminal-emulator -e neoterminal
Icon=utilities-terminal
Terminal=false
Categories=System;Utility;TerminalEmulator;
StartupNotify=true
EOF
chmod +x "$LOCAL_APPS/neoterminal.desktop"

# Create Desktop shortcut if Desktop exists
if [ -d "$DESKTOP_DIR" ]; then
    cp "$LOCAL_APPS/neoterminal.desktop" "$DESKTOP_DIR/neoterminal.desktop"
    chmod +x "$DESKTOP_DIR/neoterminal.desktop"
    echo -e "${GREEN}✓ Created shortcut on your Desktop.${NC}"
fi

echo -e "\n${BLUE}[4/4] Updating desktop database...${NC}"
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$LOCAL_APPS"
fi

echo -e "\n${GREEN}=====================================================${NC}"
echo -e "${GREEN}  ✓ NeoTerminal installed successfully!              ${NC}"
echo -e "${GREEN}=====================================================${NC}"
echo -e "You can launch NeoTerminal from:"
echo -e " 1. Your desktop shortcut icon"
echo -e " 2. Your applications menu launcher (under Utilities/System)"
echo -e " 3. Typing ${CYAN}neoterminal${NC} in any existing terminal"
echo -e "\n${YELLOW}IMPORTANT FONT SETUP:${NC}"
echo -e "To render directory and file icons correctly, please configure your terminal"
echo -e "emulator (e.g. Konsole) to use ${CYAN}Hack Nerd Font${NC}."
echo -e "\nEnjoy your new terminal wrapper!"
