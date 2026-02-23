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

## Phase 2: Database & Config ✅ COMPLETE

**Commit**: `e173373` (pushed to origin)
**Upstream Source**: `11dee2e`, `03b4f53`

### Features Integrated:
1. **Gateway SQLite Database** (`crates/bizclaw-gateway/src/db.rs`)
   - Provider CRUD operations
   - Agent CRUD operations
   - Agent-channel bindings
   - Settings storage
   - Default provider seeding (OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama, LlamaCPP, Brain)
   - Migration from agents.json support

2. **Database Integration**
   - Added rusqlite dependency with bundled feature
   - Integrated database into AppState
   - Database initialization in server startup
   - WAL mode for better concurrent performance

3. **Test Coverage**
   - 10 comprehensive tests for database operations
   - Provider CRUD tests
   - Agent CRUD tests
   - Channel binding tests
   - Settings tests
   - JSON migration tests

### Quality Checks:
- ✅ Code compiles successfully
- ✅ Code formatted with `cargo fmt`
- ✅ Database tests included
- ⏳ Full test suite (pending - long running)

### Files Modified:
- `crates/bizclaw-gateway/Cargo.toml` - Added rusqlite dependency
- `crates/bizclaw-gateway/src/lib.rs` - Exported db module
- `crates/bizclaw-gateway/src/db.rs` - New database module (783 lines)
- `crates/bizclaw-gateway/src/server.rs` - Integrated database into AppState
- `crates/bizclaw-gateway/src/routes.rs` - Updated test helper

## Phase 3: Channels ✅ COMPLETE

**Commit**: `7853bb9` (pushed to origin)
**Upstream Source**: `7936063`

### Features Integrated:
1. **Webhook Channel Configuration** (`crates/bizclaw-core/src/config.rs`)
   - Added `WebhookChannelConfig` struct
   - Supports inbound webhook secret for HMAC-SHA256 verification
   - Supports outbound URL for sending replies
   - Integrated into `ChannelConfig`

2. **Gateway API Updates** (`crates/bizclaw-gateway/src/routes.rs`)
   - Added webhook config to `/api/v1/config` endpoint
   - Masked secret display for security
   - Shows webhook enabled status and outbound URL

3. **Existing Webhook Channel**
   - Webhook channel implementation already exists in `crates/bizclaw-channels/src/webhook.rs`
   - Supports signature verification
   - Supports inbound message injection
   - Supports outbound message posting

### Quality Checks:
- ✅ Code compiles successfully
- ✅ Clippy clean with `-D warnings`
- ✅ Code formatted with `cargo fmt`
- ✅ No breaking changes

### Files Modified:
- `crates/bizclaw-core/src/config.rs` - Added WebhookChannelConfig
- `crates/bizclaw-gateway/src/routes.rs` - Added webhook to config API

## Phase 4: UI Features (NEXT)

### Target Commits:
- `7936063` - Generic webhook inbound channel

### Files to Integrate:
- Webhook channel improvements
- Config updates for webhook support

## Phase 4: UI Features (NEXT)

### Target Commits:
- `ae6d855` - Gallery page API + custom skills + agent-channel binding
- `5f6b7dc` - Gallery templates + data persistence
- `68bf9dc` - Provider/model in edit + MD→agent + business gallery templates
- `26d3c4e` - Provider/model selection per agent + rename to AI Agent

### Strategy:
- Focus on backend API changes only
- Skip dashboard.html changes (will merge separately)
- Add gallery API endpoints
- Add provider/model selection to agent CRUD

## Phase 5: Fixes & Polish (PLANNED)

### Target Commits:
- Bug fixes from upstream
- Error handling improvements
- Documentation updates

## Current Status

**Branch**: `features/upstream-sync`
**Commits Ahead**: 3 (Phase 1 + Phase 2 + Phase 3)
**Upstream Position**: `11dee2e` (latest)
**Build**: ✅ Compiles successfully
**Clippy**: ✅ Clean
**Next Action**: Phase 4 - Gallery API and provider/model selection

## Notes

- Infrastructure files (CI/CD, Docker, Cross.toml) are kept from our fork
- README.md and dashboard.html conflicts will be resolved manually at the end
- All phases maintain test coverage and clippy compliance
- Each phase is committed separately for easy rollback if needed
