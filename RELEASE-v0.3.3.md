# Release v0.3.3 - Test Build

**Release Date**: February 25, 2025
**Branch**: master
**Type**: Test Release

## Overview

This is a test release to verify the CI/CD pipeline and Docker image build process after configuring workflow permissions.

## Changes

- Added Docker image troubleshooting documentation
- Verified workflow permissions configuration
- Test build to confirm GHCR push works correctly

## What's Included

All features from v0.3.2:

### Critical Fixes
- Docker signal handling permission error resolved
- Graceful fallback when signal permissions unavailable
- Containers start successfully without root

### From v0.3.1
- Multi-tenant user management
- Database lock debugging with tracing
- WAL mode for SQLite (prevents lock errors)
- Storage cleanup automation
- Local Docker build support

## Testing

Verify the Docker image:

```bash
# Pull the image
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.3

# Test it works
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.3 --version

# Should output: bizclaw 0.3.3
```

## Build Artifacts

This release includes:

### Binaries
- Linux: x64, ARM64, ARMv7, ARMv6
- macOS: Intel, Apple Silicon
- Windows: x64

### Docker Images
- Multi-arch: amd64, arm64, armv7, armv6
- Available on GHCR: `ghcr.io/ngtrthanh/bizclaw:v0.3.3`
- Also tagged as: `ghcr.io/ngtrthanh/bizclaw:latest`

## Verification

If this build succeeds, it confirms:
- ✅ Workflow permissions are configured correctly
- ✅ GHCR push works
- ✅ Multi-arch Docker builds work
- ✅ All platforms compile successfully

---

**Full Changelog**: v0.3.2...v0.3.3
