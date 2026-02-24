# GitHub Storage Cleanup Guide

## Overview

GitHub Free plan has storage limits:
- **Artifact storage**: 500 MB
- **Actions minutes**: 2,000 per month
- **Cache storage**: 10 GB

This guide helps you manage storage by cleaning up old releases, artifacts, and caches.

## Quick Cleanup

### Option 1: Automated Workflow (Recommended)

Trigger the cleanup workflow manually:

```bash
gh workflow run cleanup-old-releases.yml
```

Or via GitHub UI:
1. Go to Actions tab
2. Select "Cleanup Old Releases" workflow
3. Click "Run workflow"
4. Choose how many releases to keep (default: 2)

### Option 2: Manual Script

**Linux/macOS:**
```bash
chmod +x scripts/cleanup-releases.sh
./scripts/cleanup-releases.sh 2  # Keep latest 2 releases
```

**Windows:**
```powershell
.\scripts\cleanup-releases.ps1 -KeepCount 2
```

## What Gets Cleaned Up

### Automated Cleanup Workflow

The workflow cleans up three types of storage:

1. **Old Releases** (keeps latest N)
   - Deletes release assets (binaries)
   - Removes associated tags
   - Frees up artifact storage

2. **Old Artifacts** (>7 days)
   - Build artifacts from CI runs
   - Temporary files from workflows
   - Automatically expires after 7 days

3. **Old Caches** (>7 days)
   - Cargo build caches
   - Dependency caches
   - Docker layer caches

### Release Workflow Auto-Cleanup

The release workflow now automatically:
- Deletes workflow artifacts after successful release
- Cleans up artifacts older than 7 days
- Runs after every release build

## Storage Best Practices

### 1. Keep Only Recent Releases

Keep only the latest 1-2 releases:
- Latest stable version
- Previous version for rollback

Delete older releases:
```bash
./scripts/cleanup-releases.sh 2
```

### 2. Enable Automatic Cleanup

The cleanup workflow runs weekly (Sunday 2 AM UTC) to:
- Keep latest 2 releases
- Delete artifacts >7 days old
- Clear old caches

### 3. Manual Cleanup When Needed

If you hit storage limits:

```bash
# Delete all but latest release
./scripts/cleanup-releases.sh 1

# Or trigger workflow
gh workflow run cleanup-old-releases.yml -f keep_latest=1
```

### 4. Optimize Release Artifacts

Current release includes:
- 6 platform binaries (~50-100 MB each)
- Docker images (stored separately)

To reduce size:
- Use compression (already enabled)
- Strip debug symbols (already enabled)
- Consider removing less-used platforms

## Monitoring Storage Usage

### Check Repository Size

```bash
gh api repos/ngtrthanh/bizclaw --jq '.size' | awk '{printf "%.2f MB\n", $1/1024}'
```

### Check Artifacts

```bash
gh api repos/ngtrthanh/bizclaw/actions/artifacts --jq '.total_count'
```

### Check Caches

```bash
gh api repos/ngtrthanh/bizclaw/actions/caches --jq '.total_count'
```

## Troubleshooting

### "Storage limit exceeded" Error

1. **Immediate fix:**
   ```bash
   ./scripts/cleanup-releases.sh 1  # Keep only latest
   ```

2. **Delete all artifacts:**
   ```bash
   gh workflow run cleanup-old-releases.yml -f keep_latest=1
   ```

3. **Wait for automatic cleanup:**
   - Artifacts expire after 7 days
   - Caches expire after 7 days

### "Cannot delete release" Error

If a release is referenced by a workflow:
1. Cancel running workflows
2. Wait for workflows to complete
3. Retry deletion

### Manual Artifact Deletion

```bash
# List all artifacts
gh api repos/ngtrthanh/bizclaw/actions/artifacts --paginate --jq '.artifacts[] | "\(.id) \(.name) \(.created_at)"'

# Delete specific artifact
gh api repos/ngtrthanh/bizclaw/actions/artifacts/ARTIFACT_ID -X DELETE
```

## Automation Schedule

### Weekly Cleanup (Sunday 2 AM UTC)
- Keeps latest 2 releases
- Deletes artifacts >7 days
- Clears old caches

### After Each Release
- Deletes workflow artifacts
- Cleans up build artifacts
- Optimizes storage

## Storage Optimization Tips

### 1. Reduce Build Frequency

Only build on:
- Tagged releases (v*)
- Manual triggers
- Critical branches

### 2. Use Smaller Artifacts

Current optimizations:
- ✅ Strip debug symbols
- ✅ Compress with tar.gz
- ✅ Use release builds
- ✅ Optimize Docker layers

### 3. Clean Up Regularly

Run cleanup:
- After each release
- Weekly via automation
- When approaching limits

### 4. Monitor Usage

Check storage weekly:
```bash
gh api repos/ngtrthanh/bizclaw --jq '.size'
```

## GitHub Storage Limits

### Free Plan
- Artifact storage: 500 MB
- Actions minutes: 2,000/month
- Cache storage: 10 GB

### Pro Plan ($4/month)
- Artifact storage: 2 GB
- Actions minutes: 3,000/month
- Cache storage: 10 GB

### Team Plan ($4/user/month)
- Artifact storage: 2 GB
- Actions minutes: 3,000/month
- Cache storage: 10 GB

## Alternative Solutions

### 1. Use External Storage

Upload releases to:
- AWS S3
- Google Cloud Storage
- Azure Blob Storage
- DigitalOcean Spaces

### 2. Self-Hosted Runners

Run builds on your own infrastructure:
- No storage limits
- Faster builds
- More control

### 3. Reduce Platforms

Build only essential platforms:
- Linux x64 (most common)
- Docker (multi-arch)
- Skip ARM variants if not needed

## Support

For issues or questions:
- GitHub Issues: https://github.com/ngtrthanh/bizclaw/issues
- Workflow Logs: https://github.com/ngtrthanh/bizclaw/actions

---

**Last Updated**: February 24, 2025
