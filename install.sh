#!/bin/bash
# ═══════════════════════════════════════════════════════════════
# BizClaw AI Agent Platform — One-Click Install
# Usage: curl -sSL https://bizclaw.vn/install.sh | bash
# Works on: Ubuntu/Debian VPS, Raspberry Pi, any Linux with systemd
# ═══════════════════════════════════════════════════════════════

set -e

REPO="https://github.com/nguyenduchoai/bizclaw.git"
INSTALL_DIR="/opt/bizclaw"
BIN_DIR="/usr/local/bin"
DATA_DIR="$HOME/.bizclaw"
SERVICE_NAME="bizclaw-platform"

echo ""
echo "  🦀 BizClaw AI Agent Platform — Installer"
echo "  ════════════════════════════════════════════"
echo ""

# Check root or sudo
if [ "$(id -u)" -ne 0 ]; then
  echo "⚠️  Please run as root or with sudo"
  echo "  sudo bash -c \"\$(curl -sSL https://bizclaw.vn/install.sh)\""
  exit 1
fi

# Detect OS
if [ -f /etc/os-release ]; then
  . /etc/os-release
  OS=$ID
else
  OS="unknown"
fi

echo "📦 OS detected: $OS ($PRETTY_NAME)"
echo "📦 Architecture: $(uname -m)"
echo ""

# ── Step 1: Install dependencies ────────────────────────────
echo "🔧 [1/5] Installing dependencies..."
if [ "$OS" = "ubuntu" ] || [ "$OS" = "debian" ]; then
  apt-get update -qq
  apt-get install -y -qq git curl build-essential pkg-config libssl-dev >/dev/null 2>&1
elif [ "$OS" = "fedora" ] || [ "$OS" = "centos" ] || [ "$OS" = "rhel" ]; then
  dnf install -y git curl gcc make openssl-devel >/dev/null 2>&1
else
  echo "⚠️  Unknown OS. Please install git, curl, gcc, openssl-dev manually."
fi
echo "  ✅ Dependencies installed"

# ── Step 2: Install Rust (if not present) ───────────────────
echo "🦀 [2/5] Checking Rust toolchain..."
if ! command -v cargo &>/dev/null; then
  echo "  📥 Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y >/dev/null 2>&1
  source "$HOME/.cargo/env"
  echo "  ✅ Rust installed: $(rustc --version)"
else
  source "$HOME/.cargo/env" 2>/dev/null || true
  echo "  ✅ Rust already installed: $(rustc --version)"
fi

# ── Step 3: Clone & build ───────────────────────────────────
echo "🔨 [3/5] Building BizClaw (this takes 2-5 minutes)..."
if [ -d "$INSTALL_DIR" ]; then
  cd "$INSTALL_DIR" && git pull origin master --quiet
else
  git clone --quiet "$REPO" "$INSTALL_DIR"
  cd "$INSTALL_DIR"
fi

cargo build --release --bin bizclaw --bin bizclaw-platform 2>&1 | tail -3
echo "  ✅ Build complete"

# ── Step 4: Install binaries ────────────────────────────────
echo "📦 [4/5] Installing binaries..."
cp "$INSTALL_DIR/target/release/bizclaw" "$BIN_DIR/bizclaw"
cp "$INSTALL_DIR/target/release/bizclaw-platform" "$BIN_DIR/bizclaw-platform"
chmod +x "$BIN_DIR/bizclaw" "$BIN_DIR/bizclaw-platform"
echo "  ✅ bizclaw → $BIN_DIR/bizclaw ($(du -h $BIN_DIR/bizclaw | cut -f1))"
echo "  ✅ bizclaw-platform → $BIN_DIR/bizclaw-platform ($(du -h $BIN_DIR/bizclaw-platform | cut -f1))"

# Create data directory
mkdir -p "$DATA_DIR"

# ── Step 5: Setup systemd service ───────────────────────────
echo "🚀 [5/5] Setting up systemd service..."
JWT_SECRET="bizclaw-$(head /dev/urandom | tr -dc a-z0-9 | head -c 16)"

cat > "/etc/systemd/system/${SERVICE_NAME}.service" << EOF
[Unit]
Description=BizClaw AI Agent Platform
After=network.target

[Service]
Type=simple
User=root
ExecStart=${BIN_DIR}/bizclaw-platform --port 3001 --bizclaw-bin ${BIN_DIR}/bizclaw --jwt-secret ${JWT_SECRET}
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=BIZCLAW_CONFIG=${DATA_DIR}/config.toml

[Install]
WantedBy=multi-user.target
EOF

systemctl daemon-reload
systemctl enable "${SERVICE_NAME}" >/dev/null 2>&1
systemctl restart "${SERVICE_NAME}"
sleep 2

# ── Done! ───────────────────────────────────────────────────
echo ""
echo "  ╔═══════════════════════════════════════════════════╗"
echo "  ║  🎉  BizClaw installed successfully!              ║"
echo "  ╠═══════════════════════════════════════════════════╣"
echo "  ║                                                   ║"
echo "  ║  Dashboard:  http://$(hostname -I | awk '{print $1}'):3001       ║"
echo "  ║  CLI:        bizclaw chat                         ║"
echo "  ║  Status:     systemctl status ${SERVICE_NAME}    ║"
echo "  ║  Logs:       journalctl -u ${SERVICE_NAME} -f    ║"
echo "  ║                                                   ║"
echo "  ║  Config:     ${DATA_DIR}/config.toml              ║"
echo "  ║  JWT Secret: ${JWT_SECRET}                        ║"
echo "  ║                                                   ║"
echo "  ╚═══════════════════════════════════════════════════╝"
echo ""
echo "  💡 Next steps:"
echo "     1. Open the dashboard in your browser"
echo "     2. Set your AI provider (OpenAI, Ollama, Brain Engine, etc.)"
echo "     3. Start chatting!"
echo ""
