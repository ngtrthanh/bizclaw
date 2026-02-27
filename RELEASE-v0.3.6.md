# Release v0.3.6

**Release Date**: February 25, 2025
**Type**: Stable Release

## Overview

This is a stable release with the Docker signal handling issue completely resolved. All features from Phase 5 upstream sync are included.

## What's Included

### Critical Fixes
- ✅ Docker signal handling fixed (tokio signal feature disabled)
- ✅ Multi-tenant user management
- ✅ Database lock debugging with tracing
- ✅ WAL mode for SQLite (prevents lock errors)

### Infrastructure
- ✅ Storage cleanup automation
- ✅ Local Docker build support
- ✅ Comprehensive troubleshooting documentation

### From Upstream (Phase 5)
- Proactive agent loop with background monitoring
- Scheduler-agent wire with notification dispatch
- Persistent scheduling with SQLite
- Workflow engine with event matching
- Plan persistence and tracking
- Gateway SQLite database for CRUD operations
- Provider/Agent management
- Agent-channel bindings
- Webhook channel configuration
- Agent update API

## Testing

```bash
# Pull and test
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.6
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.6 --version

# Run the server
docker run --rm -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:v0.3.6 serve

# Or use docker-compose
docker-compose up -d
```

## Build Artifacts

- **Linux**: x64, ARM64, ARMv7, ARMv6
- **macOS**: Intel, Apple Silicon
- **Windows**: x64
- **Docker**: Multi-arch (amd64, arm64, armv7, armv6)

## Upgrade Instructions

```bash
# Docker Compose
docker-compose pull
docker-compose up -d

# Docker Run
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.6
docker run -d --name bizclaw -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:v0.3.6 serve
```

## Technical Details

### Tokio Configuration
```toml
tokio = { 
  version = "1", 
  features = [
    "rt-multi-thread", "macros", "io-util", "io-std",
    "net", "time", "sync", "fs", "process"
  ]
}
```

Signal feature explicitly excluded to prevent Docker permission issues.

## Known Issues

- Third-party warning from `imap-proto v0.10.2` (no functional impact)

## Contributors

- Infrastructure & Fixes: @ngtrthanh
- Upstream Features: @nguyenduchoai

## Links

- [GitHub Repository](https://github.com/ngtrthanh/bizclaw)
- [Upstream Repository](https://github.com/nguyenduchoai/bizclaw)
- [CI/CD Documentation](./CI-CD-SETUP.md)
- [Docker Troubleshooting](./DOCKER-SIGNAL-HANDLING.md)

---

**Full Changelog**: v0.3.5...v0.3.6
