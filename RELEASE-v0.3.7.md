# Release v0.3.7

**Release Date**: February 27, 2026
**Type**: Critical Fix Release

## Overview

This release FINALLY fixes the Docker signal handling issue by completely removing tokio's process feature and signal dependencies. All process execution now uses `std::process::Command` wrapped in `tokio::task::spawn_blocking`.

## Critical Fixes

- ✅ **DOCKER SIGNAL FIX**: Removed tokio process feature entirely
- ✅ All command execution uses `std::process::Command` + `spawn_blocking`
- ✅ No signal-hook-registry dependency
- ✅ Works in Docker containers without permission errors

## Changes

### Process Execution Refactor
- `crates/bizclaw-runtime`: Uses `std::process::Command` with `spawn_blocking`
- `crates/bizclaw-security`: Sandbox uses blocking process execution
- `crates/bizclaw-tools`: Shell and execute_code tools use blocking execution
- `Cargo.toml`: Removed `process` from tokio features

### MCP Temporarily Disabled
- MCP (Model Context Protocol) temporarily disabled to avoid signal dependencies
- Will be re-enabled in future release with signal-free implementation
- Does not affect core functionality

## Testing

```bash
# Pull and test
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.7
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.7 --version

# Run the server
docker run --rm -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:v0.3.7 serve

# Or use docker-compose
docker-compose up -d
```

## Build Artifacts

- **Linux**: x64, ARM64, ARMv7, ARMv6
- **macOS**: Apple Silicon
- **Windows**: x64
- **Docker**: Multi-arch (amd64, arm64, armv7, armv6)

## Upgrade Instructions

```bash
# Docker Compose
docker-compose pull
docker-compose up -d

# Docker Run
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.7
docker run -d --name bizclaw -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:v0.3.7 serve
```

## Technical Details

### Tokio Configuration
```toml
tokio = { 
  version = "1", 
  features = [
    "rt-multi-thread", "macros", "io-util", "io-std",
    "net", "time", "sync", "fs"
  ],
  default-features = false
}
```

No `process` feature = No signal dependencies!

### Process Execution Pattern
```rust
// Before (v0.3.6 and earlier)
let output = tokio::process::Command::new("sh")
    .arg("-c")
    .arg(command)
    .output()
    .await?;

// After (v0.3.7)
let output = tokio::task::spawn_blocking(move || {
    std::process::Command::new("sh")
        .arg("-c")
        .arg(&command)
        .output()
})
.await
.map_err(|e| BizClawError::Tool(format!("Task join error: {}", e)))?;
```

## Known Issues

- MCP functionality temporarily disabled (will be re-enabled)
- Third-party warning from `imap-proto v0.10.2` (no functional impact)

## Contributors

- Infrastructure & Fixes: @ngtrthanh
- Upstream Features: @nguyenduchoai

## Links

- [GitHub Repository](https://github.com/ngtrthanh/bizclaw)
- [Upstream Repository](https://github.com/nguyenduchoai/bizclaw)
- [CI/CD Documentation](./CI-CD-SETUP.md)

---

**Full Changelog**: v0.3.6...v0.3.7
