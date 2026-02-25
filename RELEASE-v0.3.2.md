# Release v0.3.2 - Docker Signal Handling Fix

**Release Date**: February 25, 2025
**Branch**: master
**Type**: Bug Fix Release

## Overview

This is a critical bug fix release that resolves the Docker signal handling permission error that prevented containers from starting properly.

## Critical Fix

### Docker Signal Handling Permission Error

**Problem:**
Containers were crashing on startup with:
```
thread 'main' panicked at tokio-1.49.0/src/signal/unix.rs:72:53:
failed to create UnixStream: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
```

**Root Cause:**
Tokio's `tokio::signal::ctrl_c()` requires Unix domain socket creation, which needs specific permissions in Docker containers. Non-root users in containers don't have these permissions by default.

**Solution:**
Implemented graceful fallback signal handling:
- Tries to use standard signal handling
- Falls back to infinite sleep if permission denied
- Logs warning for debugging
- Container can still be stopped via Docker commands

**Files Changed:**
- `src/main.rs` - Added `wait_for_shutdown()` helper function

## Technical Details

### New Signal Handler

```rust
async fn wait_for_shutdown() {
    #[cfg(unix)]
    {
        match tokio::signal::ctrl_c().await {
            Ok(()) => {
                tracing::info!("Received Ctrl+C signal");
            }
            Err(e) => {
                tracing::warn!("Signal handler failed ({}), using fallback", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(u64::MAX)).await;
            }
        }
    }
    
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await.ok();
    }
}
```

### Behavior

**Before:**
- Container crashes on startup
- Application never starts
- No way to run in Docker without root

**After:**
- Container starts successfully
- Application runs normally
- Graceful shutdown via `docker stop`
- Warning logged if signal handling unavailable

## Upgrade Instructions

### From v0.3.1

Simply pull the new image or binary:

```bash
# Docker
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.2
docker-compose up -d

# Binary
wget https://github.com/ngtrthanh/bizclaw/releases/download/v0.3.2/bizclaw-x86_64-unknown-linux-gnu
chmod +x bizclaw-x86_64-unknown-linux-gnu
sudo mv bizclaw-x86_64-unknown-linux-gnu /usr/local/bin/bizclaw
```

### Docker Compose

No configuration changes needed. The fix is in the application code.

```yaml
services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:v0.3.2
    init: true  # Still recommended for proper signal forwarding
    cap_add:
      - SYS_PTRACE  # Optional, but helps
```

## What's Included from v0.3.1

All features from v0.3.1 are included:

### Upstream Sync (Phase 5)
- Multi-tenant user management
- Database lock debugging with tracing
- WAL mode for SQLite (prevents lock errors)

### Infrastructure Improvements
- Storage cleanup automation
- Local Docker build support
- Comprehensive documentation

## Testing

### Verify the Fix

```bash
# Start container
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.2 --version

# Should output: bizclaw 0.3.2

# Run with docker-compose
docker-compose up -d
docker-compose logs -f

# Should see: "Gateway server listening on..." without panic
```

### Expected Behavior

1. **Container starts successfully** - No permission denied error
2. **Application runs normally** - All features work
3. **Graceful shutdown** - `docker stop` works properly
4. **Warning in logs** (if signal handling fails) - For debugging

## Known Issues

- Third-party warning from `imap-proto v0.10.2` (no impact on functionality)
- Signal handling warning may appear in logs (expected, not an error)

## Build Artifacts

This release includes pre-built binaries for:

### Linux
- `bizclaw-x86_64-unknown-linux-gnu` - Linux x64
- `bizclaw-aarch64-unknown-linux-gnu` - Linux ARM64 (Pi 4/5)
- `bizclaw-armv7-unknown-linux-gnueabihf` - Linux ARMv7 (Pi 2/3)
- `bizclaw-arm-unknown-linux-gnueabihf` - Linux ARMv6 (Pi Zero/1)

### macOS
- `bizclaw-x86_64-apple-darwin` - macOS Intel
- `bizclaw-aarch64-apple-darwin` - macOS Apple Silicon

### Windows
- `bizclaw-x86_64-pc-windows-msvc.exe` - Windows x64

### Docker
- Multi-arch Docker images: `amd64`, `arm64`, `armv7`, `armv6`
- Available on GitHub Container Registry

## Contributors

- Infrastructure & Bug Fixes: @ngtrthanh
- Upstream features: @nguyenduchoai

## Links

- [GitHub Repository](https://github.com/ngtrthanh/bizclaw)
- [Upstream Repository](https://github.com/nguyenduchoai/bizclaw)
- [Docker Signal Handling Guide](./DOCKER-SIGNAL-HANDLING.md)
- [CI/CD Documentation](./CI-CD-SETUP.md)

---

**Full Changelog**: v0.3.1...v0.3.2
