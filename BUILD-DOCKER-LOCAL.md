# Building Docker Image Locally

## Problem

The Windows binary (`.exe`) cannot run in a Linux Docker container. You need a Linux binary to build a working Docker image.

## Solutions

### Solution 1: Use Pre-built Image (Easiest)

Use the pre-built multi-arch image from GitHub Container Registry:

```yaml
# docker-compose.yml
services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:latest
    # or specific version:
    # image: ghcr.io/ngtrthanh/bizclaw:v0.3.1
```

```bash
docker-compose up -d
```

### Solution 2: Cross-Compile with Cross (Recommended for Local Development)

Install `cross` for cross-compilation:

```powershell
# Install cross
cargo install cross

# Build Linux binary
cross build --release --target x86_64-unknown-linux-gnu

# Prepare Docker build context
New-Item -ItemType Directory -Force -Path docker-bin/linux/amd64
Copy-Item target/x86_64-unknown-linux-gnu/release/bizclaw docker-bin/linux/amd64/bizclaw

# Build Docker image
docker build --platform linux/amd64 --build-arg TARGETPLATFORM=linux/amd64 -t bizclaw:local .

# Test
docker run --rm --init --cap-add=SYS_PTRACE bizclaw:local --version
```

### Solution 3: Use WSL2 (If Available)

Build in WSL2 for native Linux binary:

```powershell
# Enter WSL
wsl

# Navigate to project (adjust path as needed)
cd /mnt/c/Users/thanh/Downloads/26022-dev/bizclaw

# Build Linux binary
cargo build --release

# Exit WSL
exit

# Copy binary
Copy-Item \\wsl$\Ubuntu\home\<username>\bizclaw\target\release\bizclaw docker-bin/linux/amd64/bizclaw

# Build Docker image
docker build --platform linux/amd64 --build-arg TARGETPLATFORM=linux/amd64 -t bizclaw:local .
```

### Solution 4: Build in Docker (Slowest but Most Reliable)

Create a multi-stage Dockerfile that builds inside Docker:

```dockerfile
# Dockerfile.build
FROM rust:1.93-bookworm as builder

WORKDIR /build
COPY . .

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/bizclaw /usr/local/bin/bizclaw

RUN useradd -m -u 1000 bizclaw && \
    mkdir -p /home/bizclaw/.bizclaw /tmp/bizclaw && \
    chown -R bizclaw:bizclaw /home/bizclaw /tmp/bizclaw && \
    chmod 755 /usr/local/bin/bizclaw

ENV TMPDIR=/tmp/bizclaw

USER bizclaw
WORKDIR /home/bizclaw

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/bizclaw", "--version"]

ENTRYPOINT ["/usr/local/bin/bizclaw"]
CMD ["--help"]
```

Build:
```powershell
docker build -f Dockerfile.build -t bizclaw:local .
```

## Quick Start Scripts

### PowerShell Script (Cross-Compile)

Save as `build-docker.ps1`:

```powershell
# Install cross if not already installed
if (!(Get-Command cross -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cross..."
    cargo install cross
}

# Build Linux binary
Write-Host "Building Linux binary..."
cross build --release --target x86_64-unknown-linux-gnu

# Prepare Docker context
Write-Host "Preparing Docker context..."
New-Item -ItemType Directory -Force -Path docker-bin/linux/amd64 | Out-Null
Copy-Item target/x86_64-unknown-linux-gnu/release/bizclaw docker-bin/linux/amd64/bizclaw -Force

# Build Docker image
Write-Host "Building Docker image..."
docker build --platform linux/amd64 --build-arg TARGETPLATFORM=linux/amd64 -t bizclaw:local .

Write-Host "✅ Done! Test with: docker run --rm --init bizclaw:local --version"
```

Run:
```powershell
.\build-docker.ps1
```

## Testing the Image

### Basic Test

```bash
# Check version
docker run --rm --init --cap-add=SYS_PTRACE bizclaw:local --version

# Show help
docker run --rm --init --cap-add=SYS_PTRACE bizclaw:local --help

# Test info command
docker run --rm --init --cap-add=SYS_PTRACE bizclaw:local info
```

### Run with Docker Compose

Update `docker-compose.yml`:

```yaml
services:
  bizclaw:
    image: bizclaw:local  # Use local image
    # ... rest of config
```

Start:
```bash
docker-compose up -d
docker-compose logs -f
```

## Troubleshooting

### "exec format error"

This means you're using a Windows binary in a Linux container. Use one of the solutions above to build a proper Linux binary.

### "cross: command not found"

Install cross:
```bash
cargo install cross
```

### "Docker daemon not running"

Start Docker Desktop.

### "Permission denied" in container

Make sure you're using the updated docker-compose.yml with:
- `init: true`
- `cap_add: [SYS_PTRACE]`

### Cross-compilation fails

Install required dependencies:
```bash
# Windows (in PowerShell as Admin)
choco install llvm

# Or use WSL2 instead
```

## Production Deployment

For production, use the pre-built multi-arch images from GitHub:

```yaml
services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:v0.3.1
    init: true
    cap_add:
      - SYS_PTRACE
    # ... rest of config
```

These images are built by GitHub Actions and support:
- linux/amd64 (x86_64)
- linux/arm64 (Raspberry Pi 4/5)
- linux/arm/v7 (Raspberry Pi 2/3)
- linux/arm/v6 (Raspberry Pi Zero/1)

## Next Steps

1. Choose a build method above
2. Build the image
3. Test with `docker run`
4. Deploy with `docker-compose up -d`
5. Check logs: `docker-compose logs -f`

---

**Last Updated**: February 24, 2025
