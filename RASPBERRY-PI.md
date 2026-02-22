# BizClaw on Raspberry Pi

BizClaw supports all Raspberry Pi models with pre-built binaries and Docker images.

## Supported Models

| Model | Architecture | Binary | Docker Platform |
|-------|-------------|--------|-----------------|
| Pi Zero, Pi 1 | ARMv6 | `bizclaw-linux-armv6` | `linux/arm/v6` |
| Pi 2, Pi 3 | ARMv7 | `bizclaw-linux-armv7` | `linux/arm/v7` |
| Pi 4, Pi 5 | ARM64 | `bizclaw-linux-arm64` | `linux/arm64` |

## Installation

### Option 1: Pre-built Binary (Recommended)

Download the appropriate binary for your Pi model:

```bash
# Detect your architecture
ARCH=$(uname -m)

# Download based on architecture
if [ "$ARCH" = "aarch64" ]; then
    # Pi 4/5 (64-bit)
    wget https://github.com/ngtrthanh/bizclaw/releases/latest/download/bizclaw-linux-arm64.tar.gz
    tar xzf bizclaw-linux-arm64.tar.gz
elif [ "$ARCH" = "armv7l" ]; then
    # Pi 2/3 (32-bit)
    wget https://github.com/ngtrthanh/bizclaw/releases/latest/download/bizclaw-linux-armv7.tar.gz
    tar xzf bizclaw-linux-armv7.tar.gz
else
    # Pi Zero/1 (ARMv6)
    wget https://github.com/ngtrthanh/bizclaw/releases/latest/download/bizclaw-linux-armv6.tar.gz
    tar xzf bizclaw-linux-armv6.tar.gz
fi

# Install
sudo mv bizclaw-linux-* /usr/local/bin/bizclaw
sudo chmod +x /usr/local/bin/bizclaw

# Verify
bizclaw --version
```

### Option 2: Docker (All Models)

Docker automatically pulls the correct image for your architecture:

```bash
# Install Docker (if not already installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER

# Run BizClaw
docker run -d \
  --name bizclaw \
  -p 8080:8080 \
  -v ~/.bizclaw:/home/bizclaw/.bizclaw \
  ghcr.io/ngtrthanh/bizclaw:latest
```

### Option 3: Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:latest
    container_name: bizclaw
    restart: unless-stopped
    ports:
      - "8080:8080"
    volumes:
      - ~/.bizclaw:/home/bizclaw/.bizclaw
    environment:
      - RUST_LOG=info
```

Run:

```bash
docker-compose up -d
```

## Performance Notes

### Pi Zero / Pi 1 (ARMv6)
- Single-core ARM11 @ 700MHz-1GHz
- 512MB RAM
- Best for: CLI tools, lightweight automation
- Not recommended for: Heavy AI workloads

### Pi 2 / Pi 3 (ARMv7)
- Quad-core Cortex-A7/A53 @ 900MHz-1.4GHz
- 1GB RAM
- Best for: Small AI agents, basic automation
- Recommended: Use swap file for memory-intensive tasks

### Pi 4 / Pi 5 (ARM64)
- Quad-core Cortex-A72/A76 @ 1.5-2.4GHz
- 2-8GB RAM
- Best for: Full AI agent capabilities, production use
- Recommended: 4GB+ RAM for optimal performance

## Memory Optimization

For Pi models with limited RAM, enable swap:

```bash
# Create 2GB swap file
sudo fallocate -l 2G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

# Make permanent
echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab
```

## Building from Source

If you want to build on the Pi itself:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/ngtrthanh/bizclaw.git
cd bizclaw
cargo build --release

# Binary will be at: target/release/bizclaw
```

**Note:** Building on Pi Zero/1 can take several hours. Pre-built binaries are recommended.

## Cross-Compilation

Build for Raspberry Pi from your development machine:

```bash
# Install cross
cargo install cross

# Build for Pi 4/5 (ARM64)
cross build --release --target aarch64-unknown-linux-gnu

# Build for Pi 2/3 (ARMv7)
cross build --release --target armv7-unknown-linux-gnueabihf

# Build for Pi Zero/1 (ARMv6)
cross build --release --target arm-unknown-linux-gnueabihf
```

## Troubleshooting

### "cannot execute binary file: Exec format error"
You downloaded the wrong architecture. Check with `uname -m` and download the correct binary.

### Out of Memory
Enable swap (see Memory Optimization above) or use a Pi model with more RAM.

### Docker pull fails
Ensure you're using Docker 19.03+ which supports multi-arch images:
```bash
docker version
```

### Slow performance
- Use ARM64 build on Pi 4/5 (not ARMv7)
- Increase swap size
- Use lighter AI models
- Reduce concurrent operations

## Support

For issues specific to Raspberry Pi, please include:
- Pi model and OS version (`cat /etc/os-release`)
- Architecture (`uname -m`)
- RAM (`free -h`)
- BizClaw version (`bizclaw --version`)
