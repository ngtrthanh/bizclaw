# Upstream Sync Status

## Overview
Syncing features from upstream `nguyenduchoai/bizclaw` to fork `ngtrthanh/bizclaw` while keeping infrastructure/CI/CD separate.

## Branch Strategy
- `master` - Production branch
- `infra/stable` - CI/CD, build system, Docker (infrastructure only)
- `features/upstream-sync` - Application features from upstream (current branch)

## Phase 1: Core Features ✅ COMPLETE

**Commit**: `9fcf836` (pushed to origin)
**Upstream Source**: `60469ec`

### Features Integrated:
1. **Proactive Agent Loop** (`crates/bizclaw-agent/src/proactive.rs`)
   - Background task monitoring
   - Channel health checks
   - Pending plan execution

2. **Scheduler-Agent Wire** (`crates/bizclaw-scheduler/src/dispatch.rs`)
   - Notification dispatch system
   - Telegram, Discord, Webhook support
   - Dashboard WebSocket integration

3. **Persistent Scheduling** (`crates/bizclaw-scheduler/src/persistence.rs`)
   - SQLite-based task persistence
   - Workflow rule cooldown tracking
   - Database migration support

4. **Workflow Engine** (`crates/bizclaw-scheduler/src/workflow.rs`)
   - Event-driven workflows
   - Keyword matching
   - Channel event handling
   - Metric threshold monitoring

5. **Plan Persistence** (`crates/bizclaw-tools/src/plan_store.rs`)
   - SQLite plan storage
   - Plan CRUD operations
   - Status tracking

### Quality Checks:
- ✅ All 92 tests passing (increased from 82)
- ✅ Clippy clean with `-D warnings`
- ✅ Code formatted with `cargo fmt`
- ✅ No new compiler warnings (except third-party imap-proto)

### Files Modified:
- `crates/bizclaw-agent/src/lib.rs` - Added proactive module
- `crates/bizclaw-gateway/src/server.rs` - Integrated proactive loop
- `crates/bizclaw-mcp/src/transport.rs` - Minor updates
- `crates/bizclaw-scheduler/src/engine.rs` - Workflow integration
- `crates/bizclaw-scheduler/src/lib.rs` - Exported new modules
- `crates/bizclaw-tools/src/lib.rs` - Added plan_store module

## Phase 2: Database & Config (NEXT)

### Remaining Upstream Commits to Review:
1. `11dee2e` - SQLite DB for providers/agents CRUD, remove ClawHub branding
2. `e0b88de` - Agent edit error handling, knowledge file upload, provider inheritance
3. `bb3b0fe` - Hybrid config persistence (dashboard sync + admin settings)
4. `03b4f53` - DB as source of truth for tenant settings

### Challenges:
- Multiple conflicts in README.md and dashboard.html
- Cargo.lock conflicts (we don't track it)
- Need manual integration approach

### Strategy:
Instead of cherry-picking, manually extract and integrate:
1. New database schema from `crates/bizclaw-gateway/src/db.rs`
2. Config persistence improvements
3. Provider/agent CRUD APIs
4. Skip README and dashboard.html (merge separately later)

## Phase 3: Channels (PLANNED)

### Target Commits:
- `7936063` - Generic webhook inbound channel

### Files to Integrate:
- Webhook channel improvements
- Config updates for webhook support

## Phase 4: UI Features (PLANNED)

### Target Commits:
- Gallery page API
- Provider selection per agent
- Agent-channel binding

### Note:
UI changes (dashboard.html) will be merged separately to avoid conflicts.

## Phase 5: Fixes & Polish (PLANNED)

### Target Commits:
- Bug fixes from upstream
- Error handling improvements
- Documentation updates

## Current Status

**Branch**: `features/upstream-sync`
**Commits Ahead**: 1 (Phase 1)
**Upstream Position**: `11dee2e` (latest)
**Tests**: 92 passing
**Clippy**: Clean
**Next Action**: Begin Phase 2 - Database & Config integration

## Notes

- Infrastructure files (CI/CD, Docker, Cross.toml) are kept from our fork
- README.md and dashboard.html conflicts will be resolved manually at the end
- All phases maintain test coverage and clippy compliance
- Each phase is committed separately for easy rollback if needed
