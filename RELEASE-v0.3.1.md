# Release v0.3.1 - Critical Bug Fixes

**Release Date**: February 24, 2025
**Branch**: master
**Upstream Sync**: Phase 5 (3 commits)

## Overview

This release integrates 3 critical bug fixes from upstream focusing on platform stability and database performance.

## What's New

### 1. Multi-Tenant User Management
- Added optional `tenant_id` parameter to user creation
- Enables proper user-tenant association
- Backward compatible with existing code

**Files Changed:**
- `src/platform_main.rs`
- `crates/bizclaw-platform/src/db.rs`

**Upstream Commit**: `d05f0e9`

### 2. Database Lock Debugging
- Added comprehensive tracing logs to login function
- Helps diagnose database lock issues in production
- Logs all DB lock/unlock operations
- Tracks password verification and token generation

**Files Changed:**
- `crates/bizclaw-platform/src/admin.rs`

**Upstream Commit**: `811974e`

### 3. WAL Mode for Platform Database
- Enabled Write-Ahead Logging (WAL) mode for SQLite
- Prevents "database is locked" errors
- Allows concurrent readers and writers
- Improved database performance under load
- Added busy_timeout and synchronous pragmas

**Files Changed:**
- `crates/bizclaw-platform/src/db.rs`

**Upstream Commit**: `ed3671e`

## Technical Details

### Database Improvements

The WAL mode configuration:
```rust
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA busy_timeout = 5000;
```

Benefits:
- Multiple readers can access the database simultaneously
- Writers don't block readers
- Better concurrency for multi-user scenarios
- Reduced lock contention

### Code Quality

- ✅ All tests passing (92+ tests)
- ✅ Clippy clean with `-D warnings`
- ✅ Code formatted with `cargo fmt`
- ✅ No breaking changes
- ✅ Backward compatible

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
- Available on Docker Hub

## Upgrade Instructions

### From v0.3.0

No special migration needed. Simply replace the binary or pull the new Docker image.

```bash
# Binary upgrade
wget https://github.com/ngtrthanh/bizclaw/releases/download/v0.3.1/bizclaw-x86_64-unknown-linux-gnu
chmod +x bizclaw-x86_64-unknown-linux-gnu
sudo mv bizclaw-x86_64-unknown-linux-gnu /usr/local/bin/bizclaw

# Docker upgrade
docker pull ngtrthanh/bizclaw:v0.3.1
docker-compose up -d
```

### Database Migration

The WAL mode is automatically enabled on first run. No manual migration required.

## Sync Status

**Commits Ahead**: 30 (our infrastructure + features)
**Commits Behind**: 96 (upstream features not yet integrated)

### Integrated So Far
- Phase 1: Core features (proactive agent, scheduler, workflows)
- Phase 2: Database & config (SQLite, providers/agents CRUD)
- Phase 3: Channels (webhook configuration)
- Phase 4: Agent updates (provider/model/system_prompt)
- Phase 5: Critical bug fixes (this release)

### Coming Next
- Provider refactoring (openai_compatible.rs)
- Bidirectional channel framework
- Security improvements and RBAC
- Additional bug fixes

## Known Issues

- Third-party warning from `imap-proto v0.10.2` (no impact on functionality)
- Some upstream features require architectural refactoring before integration

## Contributors

- Infrastructure & CI/CD: @ngtrthanh
- Upstream features: @nguyenduchoai

## Links

- [GitHub Repository](https://github.com/ngtrthanh/bizclaw)
- [Upstream Repository](https://github.com/nguyenduchoai/bizclaw)
- [CI/CD Documentation](./CI-CD-SETUP.md)
- [Upstream Sync Status](./UPSTREAM-SYNC-STATUS.md)
- [Daily Sync Workflow](./UPSTREAM-SYNC-WORKFLOW.md)

---

**Full Changelog**: v0.3.0...v0.3.1
