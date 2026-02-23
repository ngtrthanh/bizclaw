# BizClaw — API Reference

> **Version**: 0.2.0  
> **Base URL**: `http://{host}:{port}`  
> **Auth**: `X-Pairing-Code` header (or `?code=` query param for WebSocket)

---

## Authentication

All protected endpoints require the `X-Pairing-Code` header:

```
X-Pairing-Code: your-pairing-code
Content-Type: application/json
```

### Verify Pairing (Public)
```
POST /api/v1/verify-pairing
Body: {"code": "your-pairing-code"}
Response: {"ok": true}
```

---

## System

### Health Check (Public)
```
GET /health
Response: {"status": "ok"}
```

### System Info
```
GET /api/v1/info
Response: {
  "name": "BizClaw",
  "version": "0.2.0",
  "provider": "openai",
  "model": "gpt-4o-mini",
  "uptime_secs": 3600,
  "tools": 15
}
```

### System Health Check
```
GET /api/v1/health
Response: {
  "ok": true,
  "status": "healthy",
  "score": "6/7",
  "score_pct": 85,
  "checks": [
    {"name": "Config File", "status": "pass", "detail": "..."},
    {"name": "API Key", "status": "pass", "detail": "..."},
    ...
  ]
}
```

---

## Configuration

### Get Config
```
GET /api/v1/config
Response: {
  "default_provider": "openai",
  "default_model": "gpt-4o-mini",
  "api_key": "sk-***",
  "api_base_url": "",
  ...
}
```

### Update Config
```
POST /api/v1/config/update
Body: {
  "default_provider": "ollama",
  "default_model": "llama3.2",
  "api_key": "sk-...",
  "api_base_url": "http://localhost:8787/v1"
}
Response: {"ok": true, "message": "Config updated"}
```

---

## Multi-Agent

### List Agents
```
GET /api/v1/agents
Response: {
  "ok": true,
  "agents": [
    {
      "name": "CTO",
      "role": "coder",
      "description": "A coder agent",
      "provider": "ollama",
      "tools": 15,
      "messages_processed": 3,
      "conversation_length": 6,
      "is_default": true
    }
  ],
  "total": 1,
  "default": "CTO"
}
```

### Create Agent
```
POST /api/v1/agents
Body: {
  "name": "researcher",
  "role": "researcher",
  "description": "Research agent",
  "system_prompt": "You are a research specialist..."
}
Response: {"ok": true, "name": "researcher", "role": "researcher", "total_agents": 2}
```

### Update Agent
```
PUT /api/v1/agents/{name}
Body: {"role": "analyst", "description": "Updated description"}
Response: {"ok": true, "message": "Agent 'researcher' updated"}
```

### Delete Agent
```
DELETE /api/v1/agents/{name}
Response: {"ok": true, "message": "Agent 'researcher' removed"}
```

### Chat with Agent
```
POST /api/v1/agents/{name}/chat
Body: {"message": "Hello, how are you?"}
Response: {
  "ok": true,
  "agent": "CTO",
  "response": "I'm doing well! How can I help?"
}
```

### Broadcast to All Agents
```
POST /api/v1/agents/broadcast
Body: {"message": "What is 2+2?"}
Response: {
  "ok": true,
  "responses": [
    {"agent": "CTO", "ok": true, "response": "4"},
    {"agent": "researcher", "ok": true, "response": "The answer is 4."}
  ]
}
```

---

## Telegram Bot ↔ Agent

### Connect Bot
```
POST /api/v1/agents/{name}/telegram
Body: {"bot_token": "123456:ABC-DEF..."}
Response: {
  "ok": true,
  "agent": "CTO",
  "bot_username": "my_bot",
  "message": "@my_bot connected to agent 'CTO'"
}
```

### Disconnect Bot
```
DELETE /api/v1/agents/{name}/telegram
Response: {"ok": true, "message": "@my_bot disconnected from agent 'CTO'"}
```

### Bot Status
```
GET /api/v1/agents/{name}/telegram
Response: {"ok": true, "connected": true, "bot_username": "my_bot", "agent": "CTO"}
```

---

## Knowledge Base (RAG)

### Search
```
POST /api/v1/knowledge/search
Body: {"query": "how to deploy", "limit": 5}
Response: {
  "ok": true,
  "results": [
    {"doc_name": "deploy-guide.md", "content": "...", "score": 0.85, "chunk_idx": 0}
  ]
}
```

### List Documents
```
GET /api/v1/knowledge/documents
Response: {
  "ok": true,
  "documents": [{"id": 1, "name": "guide.md", "source": "api", "chunks": 5}],
  "total_docs": 1,
  "total_chunks": 5
}
```

### Add Document
```
POST /api/v1/knowledge/documents
Body: {"name": "guide.md", "content": "...", "source": "upload"}
Response: {"ok": true, "chunks": 5}
```

### Remove Document
```
DELETE /api/v1/knowledge/documents/{id}
Response: {"ok": true}
```

---

## Brain Workspace

### List Files
```
GET /api/v1/brain/files
Response: {
  "ok": true,
  "files": ["SOUL.md", "IDENTITY.md", "USER.md", "MEMORY.md"],
  "base_dir": "/root/.bizclaw/brain",
  "count": 4
}
```

### Read File
```
GET /api/v1/brain/files/{filename}
Response: {"ok": true, "filename": "SOUL.md", "content": "...", "size": 500}
```

### Write File
```
PUT /api/v1/brain/files/{filename}
Body: {"content": "# My Agent Soul\n..."}
Response: {"ok": true, "message": "Saved: SOUL.md"}
```

### AI Personalization
```
POST /api/v1/brain/personalize
Body: {
  "about_user": "I'm a developer building SaaS apps",
  "agent_vibe": "friendly and professional",
  "agent_name": "DevBot",
  "language": "en"
}
Response: {"ok": true, "saved": ["SOUL.md", "IDENTITY.md", "USER.md"]}
```

---

## WebSocket

### Connect
```
ws://{host}:{port}/ws?code=pairing-code
```

### Message Types

**Chat (send)**
```json
{"type": "chat", "content": "Hello!", "stream": true}
```

**Response (receive)**
```json
{"type": "chunk", "content": "Hello"}
{"type": "done", "full_response": "Hello! How can I help?"}
```

**Status**
```json
{"type": "status"}
→ {"type": "status", "provider": "openai", "model": "gpt-4o-mini", ...}
```

---

## Providers

### List Providers
```
GET /api/v1/providers
Response: {
  "providers": [
    {"name": "openai", "display_name": "OpenAI", "configured": true, "required_fields": ["api_key"]}
  ],
  "current": "openai"
}
```

### Ollama Models
```
GET /api/v1/ollama/models
Response: {"ok": true, "models": ["llama3.2", "qwen2.5"]}
```

### Brain Model Scan
```
GET /api/v1/brain/models
Response: {
  "ok": true,
  "models": [{"name": "tinyllama.gguf", "path": "/path/to/model", "size": "638 MB"}]
}
```
