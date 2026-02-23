# Quick Start Guide

## For Infrastructure Work

```bash
# Work on CI/CD, build system, Docker, etc.
git checkout infra/stable
git pull origin infra/stable

# Make changes
# ... edit .github/workflows/, Dockerfile, etc. ...

git add .
git commit -m "feat(infra): your change"
git push origin infra/stable

# Deploy to production
git checkout master
git merge infra/stable
git push origin master
```

## For Syncing Upstream Features

```bash
# Get latest upstream changes
git fetch upstream

# Merge to feature branch
git checkout features/upstream-sync
git merge upstream/master

# Resolve conflicts (keep infra changes)
# Test thoroughly

# Deploy to production
git checkout master
git merge features/upstream-sync
git push origin master
```

## For Releases

```bash
# Update version in Cargo.toml
# Commit changes to master

git checkout master
git tag v0.2.3
git push origin master v0.2.3

# GitHub Actions will automatically:
# - Build 6 platform binaries
# - Create GitHub release
# - Push Docker images
```

## Branch Overview

| Branch | Purpose | Merge To |
|--------|---------|----------|
| `master` | Production | Deploy from here |
| `infra/stable` | CI/CD, build, Docker | `master` |
| `features/upstream-sync` | App features from upstream | `master` |

## Current Status

**Infrastructure (infra/stable):**
- ✅ Rust 1.93
- ✅ CI/CD workflows
- ✅ Cross-compilation
- ✅ Raspberry Pi support (ARMv6/7/ARM64)
- ✅ Docker multi-arch
- ✅ All clippy warnings fixed

**Features (to sync from upstream):**
- 🔄 3-Tier Memory
- 🔄 Plan Mode
- 🔄 Agent Gallery
- 🔄 Multi-agent orchestrator

## Links

- **CI:** https://github.com/ngtrthanh/bizclaw/actions/workflows/ci.yml
- **Releases:** https://github.com/ngtrthanh/bizclaw/actions/workflows/release.yml
- **Docker:** https://github.com/ngtrthanh/bizclaw/pkgs/container/bizclaw
- **Upstream:** https://github.com/nguyenduchoai/bizclaw

## Documentation

- [BRANCHING-STRATEGY.md](BRANCHING-STRATEGY.md) - Detailed branching workflow
- [CI-CD-STATUS.md](CI-CD-STATUS.md) - CI/CD configuration status
- [CI-CD-SETUP.md](CI-CD-SETUP.md) - CI/CD setup guide
- [RASPBERRY-PI.md](RASPBERRY-PI.md) - Raspberry Pi installation
