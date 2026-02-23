# BizClaw — System Architecture Overview

> **Version**: 0.2.0  
> **Last Updated**: 2026-02-23  
> **Status**: Production  
> **Language**: Rust (100%)

---

## 1. Vision

BizClaw is a **Rust-based AI Agent Infrastructure Platform** that enables:
- Multi-tenant AI agent deployment
- Multi-channel communication (Telegram, Zalo, WhatsApp, Email, Discord)
- Local LLM inference (Brain Engine via llama.cpp FFI)
- Multi-Agent Orchestrator with agent-to-agent delegation
- 15 built-in tools + MCP support
- Personal RAG with FTS5 search
- Session context with auto-compaction

---

## 2. Architecture Diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│                        BizClaw Platform                              │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │            bizclaw-platform (Multi-Tenant Manager)           │   │
│  │  • Per-tenant config (config.toml)                           │   │
│  │  • Tenant CRUD + systemd service management                  │   │
│  │  • JWT authentication for admin panel                        │   │
│  └─────────────┬────────────────────────────────────────────────┘   │
│                │ spawns per-tenant                                    │
│  ┌─────────────▼────────────────────────────────────────────────┐   │
│  │              bizclaw (Single Tenant Instance)                │   │
│  │                                                              │   │
│  │  ┌─────────┐  ┌──────────┐  ┌──────────┐  ┌─────────────┐  │   │
│  │  │ Gateway │  │  Agent   │  │ Channels │  │   Memory    │  │   │
│  │  │  (Axum) │  │  Engine  │  │ (7 chan.) │  │ (SQLite)    │  │   │
│  │  │  38 API │  │ 15 tools │  │ Telegram │  │ FTS5 search │  │   │
│  │  │  routes │  │ MCP      │  │ Zalo     │  │ Auto-compact│  │   │
│  │  │  WS     │  │ RAG      │  │ WhatsApp │  │ Brain WS    │  │   │
│  │  └─────────┘  └──────────┘  │ Discord  │  └─────────────┘  │   │
│  │                             │ Email    │                    │   │
│  │  ┌──────────────────────┐   │ Slack    │  ┌─────────────┐  │   │
│  │  │  Multi-Agent Orch.   │   │ Web CLI  │  │  Scheduler  │  │   │
│  │  │  • Named agents      │   └──────────┘  │  Cron/Once  │  │   │
│  │  │  • Delegation        │                  │  Interval   │  │   │
│  │  │  • Broadcast         │  ┌──────────┐   └─────────────┘  │   │
│  │  │  • Telegram Bot ↔    │  │ Knowledge│                    │   │
│  │  │    Agent mapping     │  │ Base RAG │                    │   │
│  │  └──────────────────────┘  └──────────┘                    │   │
│  │                                                              │   │
│  │  ┌──────────────────────────────────────────────────────┐   │   │
│  │  │              Providers Layer                          │   │   │
│  │  │  OpenAI │ Anthropic │ Gemini │ DeepSeek │ Groq       │   │   │
│  │  │  Ollama │ Brain Engine │ llama.cpp │ CLIProxyAPI     │   │   │
│  │  └──────────────────────────────────────────────────────┘   │   │
│  └──────────────────────────────────────────────────────────────┘   │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────────┐   │
│  │                     Dashboard (SPA)                           │   │
│  │  12 pages │ i18n VI/EN │ Dark theme │ Path-based routing     │   │
│  │  Pairing code auth │ WebSocket real-time │ Responsive        │   │
│  └──────────────────────────────────────────────────────────────┘   │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 3. Crate Architecture (14 crates)

| Crate | LOC | Purpose |
|-------|-----|---------|
| `bizclaw-core` | ~600 | Config, error types, traits (Channel, Provider, Identity) |
| `bizclaw-agent` | ~800 | Agent engine, tool execution, conversation, MCP, Orchestrator |
| `bizclaw-providers` | ~1200 | OpenAI, Anthropic, Gemini, DeepSeek, Groq, Ollama |
| `bizclaw-channels` | ~2000 | Telegram, Zalo, WhatsApp, Discord, Email, Slack |
| `bizclaw-tools` | ~2500 | 15 built-in tools (plan_mode, execute_code, etc.) |
| `bizclaw-memory` | ~800 | SQLite FTS5, Brain workspace, auto-compaction |
| `bizclaw-brain` | ~500 | llama.cpp FFI for local LLM inference |
| `bizclaw-mcp` | ~400 | Model Context Protocol client (stdio transport) |
| `bizclaw-gateway` | ~3000+ | Axum HTTP server, 38 API routes, WebSocket, Dashboard |
| `bizclaw-scheduler` | ~600 | Cron/interval/once task scheduling |
| `bizclaw-knowledge` | ~400 | RAG knowledge store with vector chunks |
| `bizclaw-security` | ~200 | Input sanitization, path validation |
| `bizclaw-runtime` | ~200 | Runtime abstraction (native/Docker) |
| `bizclaw-platform` | ~1000 | Multi-tenant manager, admin panel |

**Total**: ~14,000+ LOC Rust + 2,500+ LOC HTML/JS/CSS

---

## 4. API Surface (38 Routes)

### Core
- `GET /` — Dashboard SPA
- `GET /health` — Health check
- `POST /api/v1/verify-pairing` — Pairing auth

### Config
- `GET /api/v1/info` — System info  
- `GET /api/v1/config` — Get config
- `POST /api/v1/config/update` — Update config
- `GET /api/v1/config/full` — Full config

### Providers  
- `GET /api/v1/providers` — List providers
- `GET /api/v1/ollama/models` — List Ollama models
- `GET /api/v1/brain/models` — Scan GGUF models

### Channels
- `GET /api/v1/channels` — List channels
- `POST /api/v1/channels/update` — Update channel

### WebSocket
- `GET /ws` — Real-time chat (with pairing code query param)

### Multi-Agent
- `GET /api/v1/agents` — List agents
- `POST /api/v1/agents` — Create agent
- `PUT /api/v1/agents/{name}` — Update agent
- `DELETE /api/v1/agents/{name}` — Delete agent
- `POST /api/v1/agents/{name}/chat` — Chat with agent
- `POST /api/v1/agents/broadcast` — Broadcast to all

### Telegram Bot ↔ Agent
- `POST /api/v1/agents/{name}/telegram` — Connect bot
- `DELETE /api/v1/agents/{name}/telegram` — Disconnect bot
- `GET /api/v1/agents/{name}/telegram` — Bot status

### Knowledge Base
- `POST /api/v1/knowledge/search` — Search RAG
- `GET /api/v1/knowledge/documents` — List docs
- `POST /api/v1/knowledge/documents` — Add doc
- `DELETE /api/v1/knowledge/documents/{id}` — Remove doc

### Brain Workspace
- `GET /api/v1/brain/files` — List brain files
- `GET /api/v1/brain/files/{filename}` — Read file
- `PUT /api/v1/brain/files/{filename}` — Write file
- `DELETE /api/v1/brain/files/{filename}` — Delete file
- `POST /api/v1/brain/personalize` — AI personalization

### Scheduler
- `GET /api/v1/scheduler/tasks` — List tasks
- `POST /api/v1/scheduler/tasks` — Add task
- `DELETE /api/v1/scheduler/tasks/{id}` — Remove task
- `GET /api/v1/scheduler/notifications` — Notification history

### Health
- `GET /api/v1/health` — System health check

### Webhooks
- `GET /api/v1/webhook/whatsapp` — Meta verification
- `POST /api/v1/webhook/whatsapp` — WhatsApp messages
- `POST /api/v1/zalo/qr` — Zalo QR login

---

## 5. Dashboard (12 Pages)

| Page | Path | Purpose |
|------|------|---------|
| Dashboard | `/` | System stats, health overview |
| WebChat | `/chat` | Agent chat with agent selector |
| Settings | `/settings` | Provider, model, identity config |
| Providers | `/providers` | Card-based provider config |
| Channels | `/channels` | Channel management |
| Tools | `/tools` | 15 built-in tools display |
| Brain Engine | `/brain` | Local LLM, brain workspace, health check |
| MCP Servers | `/mcp` | Model Context Protocol servers |
| Multi-Agent | `/agents` | Create/edit/delete agents, Telegram bot |
| Groups | `/groups` | Agent group chat |
| Knowledge | `/knowledge` | RAG document management |
| Config File | `/configfile` | Raw config.toml editor |

---

## 6. Security Architecture

| Layer | Protection |
|-------|-----------|
| **Auth** | Pairing code (session-based) |
| **API** | X-Pairing-Code header middleware |
| **WebSocket** | Query param `?code=` validation |
| **CORS** | Configurable via `BIZCLAW_CORS_ORIGINS` |
| **Input** | `bizclaw-security` crate (path traversal, sanitization) |
| **Secrets** | API keys in config.toml, not in URLs |
| **Multi-Tenant** | JWT for platform admin, isolated tenant configs |

---

## 7. Deployment

| Target | Method |
|--------|--------|
| **VPS** | Direct binary + systemd (`bizclaw-platform.service`) |
| **Docker** | `docker-compose.yml` with multi-stage build |
| **One-Click** | `curl -sSL https://bizclaw.vn/install.sh \| bash` |
| **Production** | bizclaw.vn (116.118.2.98), Nginx reverse proxy, subdomain routing |

### Binary Sizes
- `bizclaw`: ~13 MB (release)
- `bizclaw-platform`: ~7.9 MB (release)

---

## 8. Current Tenants

| Tenant | Subdomain | Port |
|--------|-----------|------|
| demo | demo.bizclaw.vn | 3001 |
| sales | sales.bizclaw.vn | 3002 |

---

## 9. Tech Stack Summary

| Category | Technology |
|----------|-----------|
| Language | Rust 2021 edition |
| Web Framework | Axum 0.8 |
| Database | SQLite (FTS5) |
| LLM Inference | llama.cpp (C FFI) |
| Frontend | Vanilla HTML/JS/CSS (SPA) |
| Deployment | systemd + Nginx |
| Container | Docker multi-stage |
| CI/CD | GitHub Actions (future) |
| Monitoring | `tracing` crate + systemd journal |
