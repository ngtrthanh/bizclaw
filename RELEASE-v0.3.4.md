# Release v0.3.4 - Docker Signal Fix (Final)

**Release Date**: February 25, 2025
**Branch**: master
**Type**: Critical Bug Fix

## Overview

This release **finally** fixes the Docker signal handling permission error by using a custom Tokio runtime that doesn't automatically set up signal handlers.

## Critical Fix

### Root Cause

The `#[tokio::main]` macro automatically sets up Unix signal handlers during runtime initialization, which requires permissions that non-root Docker containers don't have.

### Solution

Replaced the automatic `#[tokio::main]` macro with a custom runtime builder:

```rust
fn main() -> Result<()> {
    // Build Tokio runtime without signal handling
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    
    runtime.block_on(async_main())
}
```

### Why This Works

- **No automatic signal setup**: Custom runtime doesn't register signal handlers
- **Full functionality**: All Tokio features still work (async, timers, IO, etc.)
- **Docker compatible**: Works in containers without special permissions
- **Clean shutdown**: Container stops properly via Docker commands

## Testing

```bash
# Pull and test
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.4
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.4 --version

# Should output: bizclaw 0.3.4 (without panic!)

# Run the server
docker run --rm -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:v0.3.4 serve

# Should start successfully and listen on port 8080
```

## What's Included

All features from previous versions:

### From v0.3.2
- Multi-tenant user management
- Database lock debugging
- WAL mode for SQLite
- Storage cleanup automation

### From v0.3.3
- Docker troubleshooting documentation
- Build verification

## Changes from v0.3.3

**Files Changed:**
- `src/main.rs` - Custom Tokio runtime without signal handlers

**Lines Changed:** ~10 lines

**Impact:** Fixes Docker startup crash completely

## Upgrade Instructions

Simply pull the new image:

```bash
# Docker Compose
docker-compose pull
docker-compose up -d

# Docker Run
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.4
docker run -d --name bizclaw ghcr.io/ngtrthanh/bizclaw:v0.3.4 serve
```

No configuration changes needed!

## Verification

### Before (v0.3.3 and earlier)
```
thread 'main' panicked at tokio-1.49.0/src/signal/unix.rs:72:53:
failed to create UnixStream: Os { code: 13, kind: PermissionDenied }
```

### After (v0.3.4)
```
🦀 BizClaw v0.3.4 — Web Dashboard
   🌐 Gateway server listening on http://0.0.0.0:8080
```

## Build Artifacts

- Linux: x64, ARM64, ARMv7, ARMv6
- macOS: Intel, Apple Silicon  
- Windows: x64
- Docker: Multi-arch (amd64, arm64, armv7, armv6)

## Known Issues

- Third-party warning from `imap-proto v0.10.2` (no impact)

## Contributors

- Bug Fix: @ngtrthanh
- Upstream Features: @nguyenduchoai

---

**Full Changelog**: v0.3.3...v0.3.4

**This should be the final fix for the Docker signal handling issue!** 🎉
