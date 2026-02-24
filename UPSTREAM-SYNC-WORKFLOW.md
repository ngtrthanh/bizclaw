# Upstream Sync Workflow

## Overview

This document defines the routine workflow for syncing features from upstream `nguyenduchoai/bizclaw` while maintaining our solid infrastructure and platform base.

## Branch Strategy

```
master (production)
├── infra/stable (infrastructure: CI/CD, Docker, Cross-compilation)
└── features/upstream-sync (application features from upstream)
```

### Branch Purposes

1. **master** - Production-ready code, receives merges from both infra and features
2. **infra/stable** - Infrastructure improvements (CI/CD, build system, Docker, deployment)
3. **features/upstream-sync** - Application features cherry-picked from upstream

## Sync Workflow

### Daily Sync Routine

**Every Day (automated check at 9 AM UTC):**

The automated workflow checks upstream status and creates/updates an issue if more than 5 commits behind.

**Manual Sync Process:**

1. **Fetch Latest Upstream**
   ```bash
   git fetch upstream
   git checkout features/upstream-sync
   ```

2. **Check Commits Behind**
   ```bash
   git log --oneline upstream/master --not HEAD | wc -l
   ```

3. **Review New Commits**
   ```bash
   git log --oneline --graph upstream/master --not HEAD -20
   ```

4. **Categorize Commits**
   - 🎯 **Application Features** - Business logic, API endpoints, agent features
   - 🏗️ **Infrastructure** - CI/CD, Docker, build system (skip or adapt)
   - 🐛 **Bug Fixes** - Critical fixes to integrate
   - 📝 **Documentation** - README, docs (merge separately)
   - 🎨 **UI Changes** - dashboard.html (merge separately)

### Integration Process

#### Phase 1: Identify Target Commits

```bash
# List commits by category
git log upstream/master --not HEAD --oneline --grep="feat:"
git log upstream/master --not HEAD --oneline --grep="fix:"
```

#### Phase 2: Cherry-Pick Application Features

For each application feature commit:

```bash
# Try cherry-pick
git cherry-pick <commit-hash> --no-commit

# If conflicts:
# - Skip README.md (keep ours)
# - Skip dashboard.html (merge separately)
# - Skip CI/CD files (keep ours)
# - Integrate code changes manually
```

#### Phase 3: Manual Integration

For commits with conflicts:

1. **Extract Changes**
   ```bash
   git show <commit-hash>:path/to/file.rs > temp_file.rs
   ```

2. **Review Diff**
   ```bash
   git diff <commit-hash>^..<commit-hash> -- path/to/file.rs
   ```

3. **Apply Manually**
   - Copy relevant code sections
   - Adapt to our structure
   - Maintain our infrastructure

4. **Test**
   ```bash
   cargo check --workspace
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   cargo fmt --all
   ```

#### Phase 4: Commit & Push

```bash
git add -u
git commit -m "feat: <description> (upstream: <commit-hash>)"
git push origin features/upstream-sync
```

### Files to Always Keep (Ours)

**Infrastructure Files:**
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `Dockerfile`
- `docker-compose.yml`
- `.dockerignore`
- `Cross.toml`
- `.cargo/config.toml`
- `rust-toolchain.toml`

**Documentation (Merge Separately):**
- `README.md`
- `CI-CD-SETUP.md`
- `RASPBERRY-PI.md`
- `BRANCHING-STRATEGY.md`

**UI Files (Merge Separately):**
- `crates/bizclaw-gateway/src/dashboard.html`
- `crates/bizclaw-platform/src/admin_dashboard.html`

### Conflict Resolution Strategy

1. **Code Conflicts**
   - Prefer upstream logic for application features
   - Keep our infrastructure/platform code
   - Test thoroughly after resolution

2. **Config Conflicts**
   - Merge config additions (new fields)
   - Keep our infrastructure settings
   - Document any skipped configs

3. **Dependency Conflicts**
   - Update dependencies if safe
   - Test cross-compilation after updates
   - Keep platform-specific dependencies (rustls vs native-tls)

## Quality Gates

Before merging to master:

1. ✅ **Build Check**
   ```bash
   cargo build --workspace --release
   ```

2. ✅ **Test Suite**
   ```bash
   cargo test --workspace
   ```

3. ✅ **Clippy Clean**
   ```bash
   cargo clippy --workspace --all-targets -- -D warnings
   ```

4. ✅ **Format Check**
   ```bash
   cargo fmt --all --check
   ```

5. ✅ **CI Passes**
   - All platforms (Linux, macOS, Windows)
   - All architectures (x64, ARM64, ARMv7, ARMv6)

6. ✅ **Cross-Compilation**
   ```bash
   cross build --target aarch64-unknown-linux-gnu
   cross build --target armv7-unknown-linux-gnueabihf
   ```

## Merge Strategy

### When to Merge

- After completing a logical set of features (e.g., Phase 1-4)
- When CI passes on all platforms
- After manual testing of key features
- When no breaking changes detected

### Merge Commands

```bash
# Merge features to master
git checkout master
git merge features/upstream-sync --no-ff -m "feat: integrate upstream features (Phase 1-4)"

# Merge infrastructure improvements
git checkout master
git merge infra/stable --no-ff -m "chore: infrastructure improvements"

# Push to origin
git push origin master
```

## Tracking Progress

### Status Document

Update `UPSTREAM-SYNC-STATUS.md` after each phase:
- Commits integrated
- Features added
- Files modified
- Quality checks passed
- Known issues

### Commit Messages

Use conventional commits:
- `feat:` - New features from upstream
- `fix:` - Bug fixes from upstream
- `chore:` - Infrastructure/tooling
- `docs:` - Documentation updates
- `ci:` - CI/CD changes

Include upstream commit reference:
```
feat: add webhook channel support (upstream: 7936063)
```

## Automation

### GitHub Actions Workflow

The `.github/workflows/upstream-sync-check.yml` workflow:

**Schedule:**
- Runs daily at 9 AM UTC
- Can be manually triggered via workflow_dispatch

**Behavior:**
- Fetches upstream repository
- Counts commits behind
- Creates/updates issue if >5 commits behind
- Provides recent commit list
- Includes sync checklist

**Manual Trigger:**
```bash
# Via GitHub UI: Actions → Upstream Sync Check → Run workflow
# Or via gh CLI:
gh workflow run upstream-sync-check.yml
```

**Force Sync:**
```bash
gh workflow run upstream-sync-check.yml -f force_sync=true
```

## Current Status

**Last Sync**: 2025-02-24
**Commits Behind**: 83
**Commits Ahead**: 27
**Branch**: features/upstream-sync

**Integrated Phases:**
- ✅ Phase 1: Core features (proactive agent, scheduler, workflows)
- ✅ Phase 2: Database & config (SQLite, providers/agents CRUD)
- ✅ Phase 3: Channels (webhook configuration)
- ✅ Phase 4: Agent updates (provider/model/system_prompt)

**Pending:**
- 🔄 Phase 5: Remaining bug fixes and features
- 📝 UI updates (dashboard.html)
- 📚 Documentation sync (README.md)

## Best Practices

1. **Small, Focused Commits** - One feature per commit
2. **Test After Each Integration** - Don't accumulate untested changes
3. **Document Skipped Changes** - Note why certain commits weren't integrated
4. **Preserve Infrastructure** - Never compromise our CI/CD and build system
5. **Regular Syncs** - Weekly syncs prevent large divergence
6. **Communication** - Document decisions in commit messages and status files

## Troubleshooting

### Large Divergence (>50 commits behind)

1. Create a new sync branch
2. Batch integrate by feature area
3. Test each batch thoroughly
4. Merge in phases

### Breaking Changes

1. Identify breaking changes early
2. Create compatibility layer if needed
3. Update tests to match new behavior
4. Document migration path

### Dependency Conflicts

1. Check if dependency update is necessary
2. Test cross-compilation after update
3. Update Cargo.lock
4. Verify all platforms still build

## Resources

- [Upstream Repository](https://github.com/nguyenduchoai/bizclaw)
- [Fork Repository](https://github.com/ngtrthanh/bizclaw)
- [CI/CD Documentation](./CI-CD-SETUP.md)
- [Branching Strategy](./BRANCHING-STRATEGY.md)
