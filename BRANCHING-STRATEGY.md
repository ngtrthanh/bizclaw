# Branching Strategy

## Overview

This fork maintains a clear separation between platform/infrastructure improvements and application features to enable selective merging with upstream.

## Branch Structure

```
master (production)
├── infra/stable (platform & CI/CD)
└── features/upstream-sync (app features from upstream)
```

### Branch Purposes

#### `master` (Default Branch)
- **Purpose:** Production-ready code
- **Contains:** Merged infra + features
- **Protected:** Yes
- **Deploy from:** This branch

#### `infra/stable` (Infrastructure Branch)
- **Purpose:** Platform, CI/CD, build system improvements
- **Contains:**
  - CI/CD workflows (`.github/workflows/`)
  - Cross-compilation setup (`Cross.toml`, `.cargo/config.toml`)
  - Docker configuration (`Dockerfile`, `docker-compose.yml`)
  - Rust toolchain (`rust-toolchain.toml`)
  - Build optimizations
  - Clippy/fmt fixes
  - Dependency updates for build/infra
- **Merge to:** `master` when stable
- **Independent from:** Upstream features

#### `features/upstream-sync` (Feature Sync Branch)
- **Purpose:** Track and merge upstream application features
- **Contains:**
  - New agent capabilities
  - Channel implementations
  - Tool additions
  - Business logic
  - UI/UX improvements
- **Sync from:** `upstream/master`
- **Merge to:** `master` after testing
- **Conflict resolution:** Prefer infra changes from `infra/stable`

## Workflow

### 1. Infrastructure Improvements

```bash
# Create feature branch from infra/stable
git checkout infra/stable
git checkout -b infra/raspberry-pi-support

# Make changes
git add .
git commit -m "feat(infra): add Raspberry Pi ARMv6 support"

# Merge back to infra/stable
git checkout infra/stable
git merge infra/raspberry-pi-support

# Merge to master
git checkout master
git merge infra/stable
git push origin master
```

### 2. Syncing Upstream Features

```bash
# Update upstream
git fetch upstream

# Sync to feature branch
git checkout features/upstream-sync
git merge upstream/master

# Resolve conflicts (prefer infra changes)
# Test thoroughly

# Merge to master
git checkout master
git merge features/upstream-sync
git push origin master
```

### 3. Hotfixes

```bash
# Create hotfix from master
git checkout master
git checkout -b hotfix/critical-bug

# Fix and merge directly to master
git checkout master
git merge hotfix/critical-bug
git push origin master

# Backport to appropriate branch if needed
```

## Current State

### Infrastructure Commits (infra/stable)
- ✅ Rust 1.93 upgrade
- ✅ CI/CD workflows (ci.yml, release.yml)
- ✅ Cross-compilation setup
- ✅ Raspberry Pi support (ARMv6/7/ARM64)
- ✅ Docker multi-arch
- ✅ All clippy warnings fixed
- ✅ Line ending normalization
- ✅ OpenSSL vendoring strategy

### Upstream Features (to sync)
- 🔄 3-Tier Memory system
- 🔄 Plan Mode
- 🔄 Agent Gallery
- 🔄 Multi-agent orchestrator
- 🔄 Knowledge UI
- 🔄 MCP servers management
- 🔄 Ollama model selector
- 🔄 Dashboard improvements

## Merge Strategy

### When Merging Upstream

**Priority Order:**
1. Keep all infra changes from `infra/stable`
2. Accept feature changes from upstream
3. Resolve conflicts manually for:
   - `Cargo.toml` (merge dependencies)
   - `README.md` (merge content)
   - Source files (case-by-case)

**Files to Always Keep (Ours):**
- `.github/workflows/*`
- `Cross.toml`
- `.cargo/config.toml`
- `rust-toolchain.toml`
- `Dockerfile`
- `docker-compose.yml`
- `.dockerignore`
- `CI-CD-*.md`
- `RASPBERRY-PI.md`

**Files to Merge Carefully:**
- `Cargo.toml` - Merge dependencies, keep our version/metadata
- `README.md` - Merge features, keep our badges/links
- `src/**/*.rs` - Case-by-case review

## Benefits

✅ **Clean Separation:** Infrastructure vs features clearly separated
✅ **Selective Merging:** Choose which upstream features to adopt
✅ **Conflict Reduction:** Infra changes don't conflict with features
✅ **Easy Rollback:** Can revert feature merges without affecting infra
✅ **Independent Testing:** Test infra and features separately
✅ **Clear History:** Git log shows what changed where

## Commands Reference

```bash
# Setup branches (one-time)
git checkout -b infra/stable
git push -u origin infra/stable

git checkout master
git checkout -b features/upstream-sync
git push -u origin features/upstream-sync

# Daily workflow
git fetch upstream                    # Get upstream changes
git checkout features/upstream-sync   # Switch to feature branch
git merge upstream/master             # Merge upstream
git checkout master                   # Switch to master
git merge features/upstream-sync      # Merge features
git push origin master                # Deploy

# Infrastructure work
git checkout infra/stable             # Work on infra
# ... make changes ...
git checkout master                   # Deploy infra
git merge infra/stable
git push origin master
```

## Version Tagging

Tags are created from `master` branch only:

```bash
git checkout master
git tag v0.2.3
git push origin v0.2.3
```

This triggers the release workflow which builds all platforms.

---

**Established:** 2026-02-23
**Status:** Active
