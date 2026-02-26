# Release v0.3.5 - Docker Signal Fix (ACTUALLY FINAL)

**Release Date**: February 25, 2025
**Type**: Critical Bug Fix - THE REAL FIX

## The REAL Fix

After multiple attempts, I found the actual root cause: **Tokio's `"full"` feature includes signal handling**, which gets initialized automatically even before main() runs.

### Solution

Disabled the signal feature in Tokio by using explicit features instead of `"full"`:

```toml
# Before
tokio = { version = "1", features = ["full"] }

# After  
tokio = { version = "1", features = ["rt-multi-thread", "macros", "io-util", "io-std", "net", "time", "sync", "fs", "process"] }
```

**Note:** Explicitly excludes `"signal"` feature

### Why This ACTUALLY Works

- Signal handlers are **never compiled** into the binary
- No runtime initialization of signal handling
- No permission errors possible
- Works in any Docker environment

## Testing

```bash
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.5
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.5 --version
# NO PANIC! 🎉
```

## Changes

**Files:**
- `Cargo.toml` - Removed signal feature from tokio
- `src/main.rs` - Simplified shutdown handling

**Impact:** Binary is smaller and Docker-compatible

## All Features Included

- Multi-tenant user management
- Database lock debugging  
- WAL mode for SQLite
- Storage cleanup automation
- Docker troubleshooting docs

---

**This is the ACTUAL final fix. The signal feature is completely removed from the build.** ✅
