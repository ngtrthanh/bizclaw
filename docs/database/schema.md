# BizClaw — Database & Storage Schema

> **Last Updated**: 2026-02-23  
> **Engine**: SQLite (FTS5 enabled)

---

## 1. Storage Architecture

BizClaw uses a **file-based storage** architecture with SQLite:

```
~/.bizclaw/
├── config.toml              # Main configuration
├── .pairing_code             # Session auth code
├── memory.db                 # Agent memory (FTS5)
├── knowledge.db              # RAG knowledge store
├── scheduler/
│   └── tasks.json            # Scheduled tasks
├── brain/
│   ├── SOUL.md               # Agent personality
│   ├── IDENTITY.md           # Agent identity
│   ├── USER.md               # User profile
│   └── MEMORY.md             # Persistent memory
├── models/
│   └── *.gguf                # GGUF model files
└── cache/
    └── *.bin                 # KV cache files
```

---

## 2. Memory Database (memory.db)

### Table: conversations
```sql
CREATE TABLE conversations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  role TEXT NOT NULL,           -- 'user', 'assistant', 'system', 'tool'
  content TEXT NOT NULL,
  timestamp TEXT NOT NULL,       -- ISO 8601
  token_count INTEGER DEFAULT 0,
  importance REAL DEFAULT 0.5   -- 0.0 to 1.0 for compaction
);

CREATE INDEX idx_conversations_timestamp ON conversations(timestamp);
CREATE INDEX idx_conversations_role ON conversations(role);
```

### FTS5 Virtual Table
```sql
CREATE VIRTUAL TABLE conversations_fts USING fts5(
  content,
  content='conversations',
  content_rowid='id',
  tokenize='porter unicode61'
);
```

### Auto-Compaction Logic
- Max token budget: configurable (default 4096)
- When exceeded: summarize old messages, keep recent
- Importance weighting: higher = kept longer

---

## 3. Knowledge Database (knowledge.db)

### Table: documents
```sql
CREATE TABLE documents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  source TEXT DEFAULT 'api',     -- 'api', 'upload', 'crawl'
  created_at TEXT NOT NULL
);
```

### Table: chunks
```sql
CREATE TABLE chunks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  doc_id INTEGER NOT NULL,
  chunk_idx INTEGER NOT NULL,
  content TEXT NOT NULL,
  FOREIGN KEY (doc_id) REFERENCES documents(id) ON DELETE CASCADE
);
```

### FTS5 Virtual Table
```sql
CREATE VIRTUAL TABLE chunks_fts USING fts5(
  content,
  content='chunks',
  content_rowid='id',
  tokenize='porter unicode61'
);
```

---

## 4. Scheduler Storage (tasks.json)

```json
{
  "tasks": [
    {
      "id": "uuid-v4",
      "name": "Daily Report",
      "schedule": "cron",
      "cron_expr": "0 9 * * *",
      "action": "send_message",
      "payload": {"message": "Generate daily report"},
      "enabled": true,
      "last_run": "2026-02-23T09:00:00Z",
      "next_run": "2026-02-24T09:00:00Z"
    }
  ]
}
```

### Schedule Types
| Type | Field | Example |
|------|-------|---------|
| `cron` | `cron_expr` | `"0 9 * * *"` (daily 9am) |
| `interval` | `interval_secs` | `3600` (every hour) |
| `once` | `run_at` | `"2026-03-01T10:00:00Z"` |

---

## 5. Config Schema (config.toml)

```toml
# Provider configuration
api_key = "sk-..."
api_base_url = ""                    # Custom API proxy URL
default_provider = "openai"          # openai, anthropic, gemini, deepseek, groq, ollama, llamacpp, brain
default_model = "gpt-4o-mini"
default_temperature = 0.7

# Agent identity
[identity]
name = "BizClaw"
persona = "A friendly AI assistant"
system_prompt = "You are BizClaw, a helpful AI."

# Gateway (HTTP server)
[gateway]
port = 3000
host = "127.0.0.1"
require_pairing = true

# Brain (local LLM)
[brain]
enabled = true
model_path = "~/.bizclaw/models/tinyllama.gguf"
threads = 4
max_tokens = 256
context_length = 2048
temperature = 0.7

# Memory
[memory]
backend = "sqlite"
auto_save = true

# Channels
[channel.telegram]
enabled = true
bot_token = "123456:ABC-DEF..."
allowed_chat_ids = []

[channel.whatsapp]
enabled = false
access_token = ""
phone_number_id = ""
webhook_verify_token = ""

[channel.zalo]
enabled = false
mode = "personal"

# MCP Servers
[[mcp_servers]]
name = "filesystem"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
enabled = true
```

---

## 6. Platform Database (Multi-Tenant)

### Directory Structure per Tenant
```
/opt/bizclaw/tenants/{tenant_id}/
├── config.toml
├── memory.db
├── knowledge.db
├── brain/
└── scheduler/
```

### Platform State (in-memory + disk)
```json
{
  "tenants": [
    {
      "id": "demo",
      "subdomain": "demo.bizclaw.vn",
      "port": 3001,
      "status": "running",
      "config_path": "/opt/bizclaw/tenants/demo/config.toml"
    }
  ]
}
```
