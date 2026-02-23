# Cherry-Pick Plan from Upstream

## Analysis Date: 2026-02-23

## Features to Cherry-Pick (Application Only)

### High Priority - Core Features

#### 1. **Proactive Agent Loop** (60469ec)
- **What:** Agent can proactively initiate conversations
- **Files:** `crates/bizclaw-agent/src/proactive.rs`
- **Impact:** Major feature - autonomous agent behavior
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 2. **Workflow Engine** (60469ec)
- **What:** Task workflow orchestration
- **Files:** `crates/bizclaw-scheduler/src/workflow.rs`
- **Impact:** Major feature - complex task chains
- **Dependencies:** Scheduler updates
- **Status:** ✅ Ready to pick

#### 3. **Plan Store Persistence** (60469ec)
- **What:** Save and resume agent plans
- **Files:** `crates/bizclaw-tools/src/plan_store.rs`
- **Impact:** Major feature - plan continuity
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 4. **Scheduler Persistence** (60469ec)
- **What:** Persistent scheduled tasks across restarts
- **Files:** `crates/bizclaw-scheduler/src/persistence.rs`
- **Impact:** Major feature - reliable scheduling
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 5. **Scheduler-Agent Dispatch** (60469ec)
- **What:** Wire scheduler to trigger agent actions
- **Files:** `crates/bizclaw-scheduler/src/dispatch.rs`
- **Impact:** Major feature - automated agent triggers
- **Dependencies:** Proactive agent
- **Status:** ✅ Ready to pick

### Medium Priority - UI/UX Features

#### 6. **Agent Gallery** (ae6d855, 5f6b7dc)
- **What:** Pre-built agent templates
- **Files:** 
  - `crates/bizclaw-gateway/src/dashboard.html` (gallery UI)
  - `data/gallery-skills.json` (templates)
  - `crates/bizclaw-gateway/src/routes.rs` (API)
- **Impact:** Medium - easier agent creation
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 7. **Per-Agent Provider/Model Selection** (26d3c4e, 68bf9dc)
- **What:** Each agent can use different LLM
- **Files:** `crates/bizclaw-gateway/src/dashboard.html`
- **Impact:** Medium - flexibility
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 8. **Webhook Inbound Channel** (7936063)
- **What:** Generic webhook receiver
- **Files:** 
  - `crates/bizclaw-gateway/src/routes.rs`
  - `crates/bizclaw-core/src/config.rs`
- **Impact:** Medium - more integrations
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 9. **Hybrid Config Persistence** (03b4f53, bb3b0fe)
- **What:** DB as source of truth for settings
- **Files:**
  - `crates/bizclaw-platform/src/db.rs`
  - `crates/bizclaw-platform/src/tenant.rs`
  - `crates/bizclaw-platform/src/admin.rs`
- **Impact:** Medium - better multi-tenant
- **Dependencies:** None
- **Status:** ✅ Ready to pick

### Low Priority - Fixes & Polish

#### 10. **Dashboard Fixes** (3f1f2e3, 3d15ca5, 005d821)
- **What:** Various UI bug fixes
- **Files:** `crates/bizclaw-gateway/src/dashboard.html`
- **Impact:** Low - polish
- **Dependencies:** None
- **Status:** ✅ Ready to pick

#### 11. **Group Chat Improvements** (32d706a, f9e28a6)
- **What:** Group chat UI fixes
- **Files:** `crates/bizclaw-gateway/src/dashboard.html`
- **Impact:** Low - polish
- **Dependencies:** None
- **Status:** ✅ Ready to pick

## Features to SKIP (Infrastructure/Conflicts)

### ❌ Skip - Infrastructure Changes
- **783b9bd** - ARM64 cross-compile (we have better)
- **afcd086** - Cross-platform build fixes (we have better)
- **a3be3fd** - cargo fmt (already done)
- **1a35560** - CI/CD (we have better)
- **44fb610** - Unit tests + CI (we have better)

### ❌ Skip - Dependency Conflicts
- **28549bf** - Unified OpenAI providers (major refactor, review separately)
- **9096a1c** - Provider registry refactor (major refactor, review separately)

## Cherry-Pick Order (Recommended)

### Phase 1: Core Features (No UI)
```bash
git checkout features/upstream-sync
git cherry-pick 60469ec  # 5 core features bundle
# Files to keep:
# - crates/bizclaw-agent/src/proactive.rs
# - crates/bizclaw-scheduler/src/{dispatch,persistence,workflow}.rs
# - crates/bizclaw-tools/src/plan_store.rs
# Skip: README.md, dashboard.html changes
```

### Phase 2: Database & Config
```bash
git cherry-pick 03b4f53  # Hybrid config persistence
git cherry-pick bb3b0fe  # Admin settings UI
# Files to keep:
# - crates/bizclaw-platform/src/{db,tenant,admin}.rs
# Skip: README.md changes
```

### Phase 3: Channels
```bash
git cherry-pick 7936063  # Webhook inbound
# Files to keep:
# - crates/bizclaw-gateway/src/routes.rs (webhook parts)
# - crates/bizclaw-core/src/config.rs (webhook config)
# Skip: dashboard.html changes (merge separately)
```

### Phase 4: UI Features
```bash
git cherry-pick ae6d855  # Agent Gallery API
git cherry-pick 5f6b7dc  # Gallery templates
git cherry-pick 26d3c4e  # Per-agent provider selection
git cherry-pick 68bf9dc  # Provider/model in edit
# Files to keep:
# - crates/bizclaw-gateway/src/{routes,server}.rs
# - data/gallery-skills.json
# - crates/bizclaw-gateway/src/dashboard.html (merge carefully)
# Skip: README.md changes
```

### Phase 5: Fixes & Polish
```bash
git cherry-pick 3f1f2e3  # Dashboard fixes
git cherry-pick 3d15ca5  # Bug fixes
git cherry-pick 005d821  # System prompt textarea
git cherry-pick 32d706a  # Group chat fixes
# Files to keep:
# - crates/bizclaw-gateway/src/dashboard.html (merge carefully)
# Skip: README.md changes
```

## Dependency Updates Needed

After cherry-picking, check and update if needed:

```toml
# In crates/bizclaw-scheduler/Cargo.toml
# May need: serde_json, tokio features

# In crates/bizclaw-tools/Cargo.toml  
# May need: serde_json for plan storage

# In crates/bizclaw-gateway/Cargo.toml
# May need: additional axum features
```

## Testing Plan

After each phase:
1. `cargo check --workspace`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`
4. `cargo fmt --all`

## Conflict Resolution Strategy

When conflicts occur:

**Always Keep (Ours):**
- `.github/workflows/*`
- `Cross.toml`
- `.cargo/config.toml`
- `rust-toolchain.toml`
- `Dockerfile`
- `docker-compose.yml`

**Merge Carefully:**
- `Cargo.toml` - Merge dependencies, keep our version
- `crates/*/Cargo.toml` - Merge dependencies
- `crates/bizclaw-gateway/src/dashboard.html` - Merge UI changes
- `crates/bizclaw-gateway/src/routes.rs` - Merge API endpoints

**Skip:**
- `README.md` - Keep ours, cherry-pick features manually later
- Any CI/CD related changes

## Estimated Impact

**Lines Added:** ~3,500 (application code only)
**Lines Changed:** ~500 (integration points)
**New Files:** ~8 (proactive.rs, workflow.rs, persistence.rs, etc.)
**Modified Files:** ~15 (mostly adding features)

**Risk Level:** Medium
- Core features are well-isolated
- UI changes need careful merging
- Database changes are additive

## Next Steps

1. Create backup branch: `git branch backup-before-cherry-pick`
2. Start with Phase 1 (core features)
3. Test thoroughly after each phase
4. Commit after each successful phase
5. If issues arise, `git cherry-pick --abort` and review

---

**Created:** 2026-02-23
**Status:** Ready to execute
