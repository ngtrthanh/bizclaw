# Release Notes - v0.3.0

**Release Date**: February 24, 2025
**Tag**: v0.3.0
**Branch**: master

## Overview

This release integrates major features from upstream (Phase 1-4) while maintaining our solid infrastructure base. It includes proactive agent capabilities, advanced scheduling, database-backed CRUD operations, and webhook support.

## 🎯 Major Features

### Phase 1: Core Features

**Proactive Agent Loop**
- Background monitoring of agent health and status
- Automatic task execution from pending plans
- Channel health checks
- Location: `crates/bizclaw-agent/src/proactive.rs`

**Scheduler-Agent Wire**
- Notification dispatch system (Telegram, Discord, Webhook, Dashboard)
- Priority-based notifications (Urgent, High, Normal, Low)
- HMAC-SHA256 signature verification for webhooks
- Location: `crates/bizclaw-scheduler/src/dispatch.rs`

**Persistent Scheduling**
- SQLite-based task persistence
- Workflow rule cooldown tracking
- Database migration support
- Location: `crates/bizclaw-scheduler/src/persistence.rs`

**Workflow Engine**
- Event-driven workflows with multiple trigger types
- Keyword matching with "any" or "all" modes
- Channel event handling
- Metric threshold monitoring
- Location: `crates/bizclaw-scheduler/src/workflow.rs`

**Plan Persistence**
- SQLite plan storage with CRUD operations
- Plan status tracking (pending, in_progress, completed, failed)
- Automatic plan execution integration
- Location: `crates/bizclaw-tools/src/plan_store.rs`

### Phase 2: Database & Configuration

**Gateway SQLite Database**
- Comprehensive database module for persistent storage
- WAL mode for better concurrent performance
- Automatic schema migrations
- Location: `crates/bizclaw-gateway/src/db.rs`

**Provider Management**
- CRUD operations for AI providers
- 8 default providers seeded automatically:
  - OpenAI (gpt-4o, gpt-4o-mini, o1-mini, o3-mini)
  - Anthropic (claude-sonnet-4, claude-3.5-sonnet, claude-3-haiku)
  - Gemini (gemini-2.5-pro, gemini-2.5-flash, gemini-2.0-flash)
  - DeepSeek (deepseek-chat, deepseek-reasoner)
  - Groq (llama-3.3-70b, mixtral-8x7b)
  - Ollama (llama3.2, qwen3, phi-4, gemma2)
  - LlamaCPP (server endpoint)
  - Brain (tinyllama-1.1b, phi-2, llama-3.2-1b)
- API key and base URL management
- Active provider tracking

**Agent Management**
- Database-backed agent CRUD
- Provider and model selection per agent
- System prompt customization
- Agent-channel bindings
- Metadata persistence

**Settings Storage**
- Key-value settings storage
- Automatic timestamp tracking
- Migration from agents.json support

### Phase 3: Channels

**Webhook Channel Configuration**
- Generic webhook support for external integrations
- HMAC-SHA256 signature verification
- Outbound URL for sending replies
- Integration with Zapier, n8n, custom APIs
- Location: `crates/bizclaw-core/src/config.rs`

**Gateway API Updates**
- Webhook config exposed in `/api/v1/config` endpoint
- Masked secret display for security
- Configuration status reporting

### Phase 4: Agent Updates

**Agent Update API**
- `PUT /api/v1/agents/{name}` endpoint
- Update role and description
- Update provider, model, and system_prompt
- Automatic agent recreation on config changes
- Preserves existing values when not specified
- Location: `crates/bizclaw-gateway/src/routes.rs`

## 🏗️ Infrastructure Improvements

**CI/CD Enhancements**
- CI workflow enabled for feature branches
- Runs on `master`, `features/upstream-sync`, `infra/stable`
- Multi-platform testing (Linux, macOS, Windows)
- Automated quality gates

**Documentation**
- `UPSTREAM-SYNC-WORKFLOW.md` - Routine sync process
- `UPSTREAM-SYNC-STATUS.md` - Current sync status
- `SYNC-SUMMARY.md` - Executive summary
- Comprehensive workflow for ongoing syncs

## 📊 Statistics

**Code Changes:**
- 25 files changed
- 3,724 insertions
- 24 deletions
- 10 new files created

**Test Coverage:**
- 92+ tests passing
- 10 new database tests
- 12 new scheduler tests
- 7 new agent tests

**Quality Metrics:**
- ✅ Clippy clean with `-D warnings`
- ✅ All platforms compile successfully
- ✅ Cross-compilation working (x64, ARM64, ARMv7, ARMv6)
- ✅ Code properly formatted

## 🔧 Technical Details

### New Dependencies

**Gateway:**
- `rusqlite = { version = "0.32", features = ["bundled"] }`

**Scheduler:**
- `rusqlite = { version = "0.32", features = ["bundled"] }`
- `chrono = { workspace = true }`

**Tools:**
- `rusqlite = { version = "0.32", features = ["bundled"] }`
- `chrono = { workspace = true }`

### Database Schema

**Providers Table:**
```sql
CREATE TABLE providers (
    name TEXT PRIMARY KEY,
    provider_type TEXT DEFAULT 'cloud',
    api_key TEXT DEFAULT '',
    base_url TEXT DEFAULT '',
    models_json TEXT DEFAULT '[]',
    is_active INTEGER DEFAULT 0,
    enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
```

**Agents Table:**
```sql
CREATE TABLE agents (
    name TEXT PRIMARY KEY,
    role TEXT DEFAULT 'assistant',
    description TEXT DEFAULT '',
    provider TEXT DEFAULT '',
    model TEXT DEFAULT '',
    system_prompt TEXT DEFAULT '',
    enabled INTEGER DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now')),
    updated_at TEXT DEFAULT (datetime('now'))
);
```

**Agent Channels Table:**
```sql
CREATE TABLE agent_channels (
    agent_name TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    instance_id TEXT DEFAULT '',
    created_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY (agent_name, channel_type, instance_id)
);
```

**Settings Table:**
```sql
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT DEFAULT '',
    updated_at TEXT DEFAULT (datetime('now'))
);
```

## 🔄 Upstream Integration

**Commits Integrated:**
- `60469ec` - Phase 1: Core features bundle
- `03b4f53` - Phase 2: DB as source of truth
- `11dee2e` - Phase 2: SQLite DB for providers/agents CRUD
- `7936063` - Phase 3: Webhook channel
- `bd6e906` - Phase 4: Agent update with provider/model

**Commits Remaining:**
- 83 commits behind upstream
- Primarily UI changes, documentation, and additional features
- Will be integrated in future releases

## 🚀 Migration Guide

### For Existing Users

1. **Database Initialization**
   - Gateway database will be created automatically at `~/.bizclaw/gateway.db`
   - Default providers will be seeded on first run
   - Existing agents.json can be migrated using the API

2. **Configuration Updates**
   - Add webhook configuration if needed:
   ```toml
   [channel.webhook]
   enabled = true
   secret = "your-webhook-secret"
   outbound_url = "https://your-api.com/webhook"
   ```

3. **API Changes**
   - New endpoint: `PUT /api/v1/agents/{name}` for updating agents
   - New endpoint: `GET /api/v1/providers` for listing providers
   - Webhook config now available in `/api/v1/config`

### Breaking Changes

None. This release is fully backward compatible.

## 📝 Known Issues

- Third-party warning from `imap-proto v0.10.2` (will be fixed in future Rust version)
- Dashboard UI not yet updated (will be merged separately)
- README.md not yet synced with upstream (will be merged separately)

## 🔮 What's Next

### v0.3.1 (Planned)
- Additional bug fixes from upstream
- Knowledge file upload improvements
- Channel binding fixes
- Chat history persistence

### v0.4.0 (Planned)
- Gallery API and templates
- Provider inheritance
- UI updates (dashboard.html)
- Documentation sync

### Ongoing
- Weekly upstream syncs (5-10 commits per week)
- Continuous integration of application features
- Maintenance of infrastructure independence

## 🙏 Acknowledgments

- Upstream repository: [nguyenduchoai/bizclaw](https://github.com/nguyenduchoai/bizclaw)
- Infrastructure team for maintaining solid build system
- All contributors to the upstream project

## 📚 Resources

- [Upstream Sync Workflow](./UPSTREAM-SYNC-WORKFLOW.md)
- [Upstream Sync Status](./UPSTREAM-SYNC-STATUS.md)
- [Sync Summary](./SYNC-SUMMARY.md)
- [CI/CD Setup](./CI-CD-SETUP.md)
- [Raspberry Pi Support](./RASPBERRY-PI.md)

---

**Full Changelog**: v0.2.2...v0.3.0
**Download**: [GitHub Releases](https://github.com/ngtrthanh/bizclaw/releases/tag/v0.3.0)
