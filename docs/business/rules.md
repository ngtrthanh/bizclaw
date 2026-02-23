# BizClaw — Business Rules & Logic

> **Last Updated**: 2026-02-23

---

## 1. Agent Processing Pipeline

```
User Message → Agent.process()
  │
  ├── 1. Build System Prompt (Identity + Brain SOUL.md/IDENTITY.md + USER.md)
  ├── 2. Check Context Budget → Auto-compact if > 70%
  ├── 3. Append to Conversation History
  ├── 4. Send to Provider (OpenAI/Ollama/etc.)
  │     └── Include tool definitions
  ├── 5. Handle Tool Calls (if any)
  │     ├── Execute tool → get result
  │     ├── Append tool result to conversation
  │     └── Re-send to provider (tool loop, max 5 iterations)
  ├── 6. Get Final Response
  ├── 7. Save to Memory
  └── 8. Return Response
```

### Tool Execution Rules
- Max tool calls per turn: 5
- Tools have `required_approval` flag → if true, ask user first
- Tool errors are caught and returned as tool results
- Dangerous tools (execute_code, file_write) require explicit config

---

## 2. Multi-Agent Rules

### Agent Creation
- Name must be unique within orchestrator
- First agent created becomes default
- Each agent gets independent conversation history

### Agent Routing
- `send_to(name, msg)` → specific agent
- `send(msg)` → default agent
- `broadcast(msg)` → all active agents
- `delegate(from, to, task)` → agent-to-agent delegation

### Telegram Bot Mapping
- One bot per agent (1:1)
- Reconnect: auto-disconnects old, starts new
- Message flow: Telegram → polling → agent.process() → reply
- Typing indicator sent during processing

---

## 3. Memory Auto-Compaction

### Trigger Conditions
- Context token count > 70% of max budget
- Manual via `/reset` command

### Compaction Algorithm
1. Calculate current token usage
2. If > 70% budget:
   a. Score messages by recency × importance
   b. Keep most recent N messages (important ones)
   c. Summarize older messages into single context
   d. Persist important facts to MEMORY.md
   e. Log to `memory/YYYY-MM-DD.md`

---

## 4. Provider Selection Logic

```
Priority:
1. CLIProxyAPI (if api_base_url set) → route as OpenAI with custom URL
2. Explicit provider from config → use directly
3. Ollama (if locally available at :11434) → auto-detect
4. Brain Engine (if model file exists) → local LLM
5. Fallback → error "No provider configured"
```

### Provider Fallback Chain
```toml
# In config.toml
[brain.fallback]
provider = "openai"
model = "gpt-4o-mini"
```

---

## 5. Security Rules

### Pairing Code Auth
- If `require_pairing = true` → all API calls need pairing code
- Code sources (priority):
  1. `BIZCLAW_PAIRING_CODE` env var
  2. `~/.bizclaw/.pairing_code` file
- No code configured → allow all (dev mode)

### Path Security
- Brain files: whitelist `.md` extension only
- No directory traversal (`../`) allowed
- Forbidden paths: `/etc`, `/root`, `/proc`, `~/.ssh`

### CORS
- Dev: allow all origins
- Prod: `BIZCLAW_CORS_ORIGINS=https://bizclaw.vn` env var

---

## 6. Channel Rules

### Telegram
- Long polling (30s timeout)
- Skip bot messages (from.is_bot = true)
- Group vs DM detection via chat_type
- Markdown formatting in responses

### Zalo
- Personal mode: cookie-based auth
- OA mode: OAuth + API keys
- Rate limits: 20 msg/min, 200 msg/hour
- Allowlist: block_strangers = true default

### WebSocket
- Ping/pong keepalive every 25s
- Reconnect with exponential backoff (max 30s)
- Streaming responses via `chunk` messages

---

## 7. Scheduler Rules

### Task Types
| Type | Trigger | Repeat |
|------|---------|--------|
| `cron` | Cron expression | Repeating |
| `interval` | Every N seconds | Repeating |
| `once` | Specific datetime | Single |

### Execution
- Background loop checks every 30 seconds
- On match: execute action
- Failed actions: logged, no retry (currently)
- Notifications stored in history

---

## 8. Knowledge Base Rules

### Document Ingestion
- Split into chunks (~500 chars each)
- Index via FTS5 for full-text search
- Source tracking (api, upload, crawl)

### Search
- FTS5 `MATCH` query
- BM25 ranking
- Limit configurable (default 5)
- Results include: doc_name, content, score, chunk_idx
