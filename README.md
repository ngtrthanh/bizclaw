# ⚡ BizClaw

> **Hạ tầng AI Agent nhanh, module hoá — viết hoàn toàn bằng Rust.**

BizClaw là nền tảng AI Agent kiến trúc trait-driven, có thể chạy **mọi nơi** — từ Raspberry Pi đến cloud server. Hỗ trợ nhiều LLM provider, kênh giao tiếp, và công cụ thông qua kiến trúc thống nhất, hoán đổi được.

[![Rust](https://img.shields.io/badge/Rust-100%25-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-113%20passing-brightgreen)]()
[![Crates](https://img.shields.io/badge/crates-14-success)]()
[![LOC](https://img.shields.io/badge/lines-32781-informational)]()

<!-- AUTO-GENERATED STATS — updated 2026-02-24 @ eaec8d8 -->

---

## 🇻🇳 Tiếng Việt

### 🚀 100% Tự Host — Không phụ thuộc Cloud

- **100% Độc lập:** Clone về là chạy — laptop, VPS, hay Raspberry Pi. Không token khoá, không telemetry.
- **Dữ liệu nội bộ:** Chat history, cấu hình lưu SQLite local. API Keys mã hoá khi cần.
- **Offline AI:** Brain Engine chạy GGUF models offline (Llama, DeepSeek). Hoặc kết nối Ollama trên LAN.

### 🎯 Tính năng chính

| Hạng mục | Chi tiết |
|----------|----------|
| **🔌 15 Providers** | OpenAI, Anthropic, Gemini, DeepSeek, Groq, OpenRouter, Together, MiniMax, xAI (Grok), Mistral, Ollama, llama.cpp, Brain Engine, CLIProxy, vLLM + custom endpoint |
| **💬 9 Channels** | CLI, Telegram, Discord, Email (IMAP/SMTP), Webhook, WhatsApp, Zalo (Personal + Official) |
| **🛠️ 13 Tools** | Shell, File, Edit File, Glob, Grep, Web Search, HTTP Request, Config Manager, Execute Code (9 ngôn ngữ), Plan Mode, Group Summarizer, Calendar, Document Reader, Memory Search, Session Context |
| **🔗 MCP** | Model Context Protocol client — kết nối MCP servers bên ngoài, mở rộng tools không giới hạn |
| **🏢 Multi-Tenant** | Admin Platform, JWT Auth, Tenant Manager, Pairing Codes, Audit Log, Per-tenant SQLite DB |
| **🌐 Web Dashboard** | 11 trang UI (VI/EN), WebSocket real-time, chat, agents, providers, gallery, channels, brain, knowledge, scheduler, settings |
| **🤖 51 Agent Templates** | 13 danh mục nghiệp vụ, system prompt chuyên sâu, cài 1 click |
| **👥 Group Chat** | Tạo nhóm agent cộng tác — gửi 1 câu hỏi, tất cả agent trong nhóm phản hồi |
| **🧠 3-Tier Memory** | Brain workspace (SOUL.md/MEMORY.md), Daily auto-compaction, FTS5 search |
| **📚 Knowledge RAG** | Upload documents → vector search, relevance scoring |
| **⏰ Scheduler** | Tác vụ hẹn giờ, agent tự chạy background |
| **💾 Persistence** | SQLite gateway.db (providers, agents, channels), agents.json backup, auto-restore |
| **🧠 Brain Engine** | GGUF inference: mmap, quantization, Flash Attention, SIMD (ARM NEON, x86 SSE2/AVX2) |
| **🔒 Security** | Command allowlist, AES-256, HMAC-SHA256, JWT + bcrypt, CORS, rate limiting |

### 🤖 Agent Gallery — 51 Mẫu Nghiệp vụ

Cài đặt agent chuyên biệt chỉ 1 click. Mỗi agent có **system prompt** tích hợp skill chuyên sâu cho doanh nghiệp Việt Nam:

| Danh mục | Số lượng | Ví dụ |
|----------|----------|-------|
| 🧑‍💼 **HR** | 5 | Tuyển dụng, Onboarding, Lương & Phúc lợi, KPI, Nội quy |
| 💰 **Sales** | 5 | CRM, Báo giá, Doanh số, Telesales, Đối tác |
| 📊 **Finance** | 5 | Kế toán, Thuế, Dòng tiền, Hóa đơn, Kiểm soát nội bộ |
| 🏭 **Operations** | 5 | Kho, Mua hàng, Vận chuyển, QC, Bảo trì |
| ⚖️ **Legal** | 4 | Hợp đồng, Tuân thủ, Sở hữu trí tuệ, Tranh chấp |
| 📞 **Customer Service** | 3 | Hỗ trợ KH, Ticket, CSAT & Feedback |
| 📣 **Marketing** | 5 | Content, SEO, Ads, Social Media, Thương hiệu |
| 🛒 **E-commerce** | 3 | Sản phẩm, Đơn hàng, Sàn TMĐT |
| 💼 **Management** | 5 | Họp, Báo cáo, Chiến lược, Dự án, OKR |
| 📝 **Admin** | 3 | Văn thư, Tài sản, Công tác phí |
| 💻 **IT** | 3 | Helpdesk, An ninh mạng, Hạ tầng |
| 📧 **Business** | 3 | Email, Dịch thuật, Phân tích dữ liệu |
| 🎓 **Training** | 2 | Đào tạo, SOP |

### 💰 Tiết kiệm token — Mỗi Agent chọn Provider riêng

Mỗi agent có thể chọn nhà cung cấp & mô hình riêng → tiết kiệm 60-80% chi phí:

\`\`\`
Agent "Dịch thuật"     → Ollama/qwen3 (miễn phí, local)
Agent "Full-Stack Dev" → Anthropic/claude-sonnet-4 (mạnh)
Agent "Social Media"   → Gemini/flash (nhanh, rẻ)
Agent "Kế toán"        → DeepSeek/chat (giá tốt)
Agent "Helpdesk"       → Groq/llama-3.3-70b (nhanh nhất)
\`\`\`

### 👥 Group Chat — Đội ngũ Agent cộng tác

Tạo nhóm nhiều agent cùng nhà cung cấp khác nhau làm việc cùng lúc. Gửi 1 câu hỏi → tất cả agent trong nhóm phản hồi theo chuyên môn.

### 🏗️ Kiến trúc

```
┌──────────────────────────────────────────────────────────┐
│              bizclaw-platform (Admin)                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐                    │
│  │ Tenant 1│ │ Tenant 2│ │ Tenant N│  ← JWT + Audit Log │
│  └────┬────┘ └────┬────┘ └────┬────┘                    │
│       └───────────┼───────────┘                          │
│                   ▼                                      │
│            bizclaw (Gateway)                             │
│  ┌────────────────────────────────────────┐              │
│  │ Axum HTTP + WebSocket + Dashboard      │              │
│  │ SQLite gateway.db (per-tenant)         │              │
│  └────────────────┬───────────────────────┘              │
│    ┌──────────────┼──────────────┐                       │
│    ▼              ▼              ▼                       │
│  bizclaw-agent  bizclaw-agent  bizclaw-agent             │
│  (Orchestrator manages N agents)                         │
│    ┌──────────────┼──────────────┐                       │
│    ▼              ▼              ▼                       │
│ 15 Providers   9 Channels    13 Tools + MCP              │
│    ▼              ▼              ▼                       │
│ Memory         Security      Knowledge                   │
│  (SQLite+FTS5) (Allowlist)   (RAG+FTS5)                  │
│    ▼                                                     │
│ Brain Engine (GGUF+SIMD) — offline inference             │
└──────────────────────────────────────────────────────────┘
```

### 🚀 Bắt đầu nhanh

```bash
# Clone và build
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw
cargo build --release

# Cài đặt (wizard tương tác)
./target/release/bizclaw init

# Chat ngay
./target/release/bizclaw agent --interactive

# Mở Web Dashboard
./target/release/bizclaw serve
```

### 🏢 Chế độ triển khai

| Mode | Binary | Use Case |
|------|--------|----------|
| **Standalone** | `bizclaw` only | 1 bot, cá nhân, test |
| **Platform** | `bizclaw` + `bizclaw-platform` | Nhiều bots, agency, production |

**Platform mode** cung cấp:
- Admin Dashboard tại `/admin/` — quản lý tenants, users, audit log
- Mỗi tenant có subdomain riêng (demo.bizclaw.vn, sales.bizclaw.vn)
- JWT authentication + per-tenant SQLite DB

### 🔗 MCP (Model Context Protocol) Support

BizClaw hỗ trợ kết nối **MCP Servers** — mở rộng tools không giới hạn mà không cần rebuild:

```toml
# config.toml
[[mcp_servers]]
name = "github"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-github"]

[[mcp_servers]]
name = "database"
command = "npx"
args = ["-y", "@modelcontextprotocol/server-postgres"]
env = { DATABASE_URL = "postgresql://..." }
```

### 🧠 Ollama / Brain Engine — Chạy AI Offline

Ollama models được **dùng chung** giữa tất cả tenants. Pull 1 lần → tất cả dùng được.

```bash
curl -fsSL https://ollama.ai/install.sh | sh
ollama pull llama3.2      # ~3.8GB
ollama pull qwen3         # ~4.7GB
```

### 📦 Crate Map

| Crate | Mô tả | Status |
|-------|--------|--------|
| `bizclaw-core` | Traits, types, config, errors | ✅ |
| `bizclaw-brain` | GGUF inference + SIMD (ARM NEON, x86 AVX2) | ✅ |
| `bizclaw-providers` | 15 LLM providers (OpenAI-compatible unified) | ✅ |
| `bizclaw-channels` | 9 channel types (CLI, Telegram, Discord, Email, Webhook, WhatsApp, Zalo) | ✅ |
| `bizclaw-memory` | SQLite + FTS5, Brain workspace, daily auto-compaction | ✅ |
| `bizclaw-tools` | 13 native tools + MCP bridge | ✅ |
| `bizclaw-mcp` | MCP client (JSON-RPC 2.0 via stdio) | ✅ |
| `bizclaw-security` | AES-256, Command allowlist, Sandbox | ✅ |
| `bizclaw-agent` | Agent loop, tool calling (max 3 rounds), context management | ✅ |
| `bizclaw-gateway` | Axum HTTP + WS + Dashboard (11 pages, i18n VI/EN) | ✅ |
| `bizclaw-knowledge` | Knowledge RAG with FTS5, document chunking | ✅ |
| `bizclaw-scheduler` | Scheduled tasks, agent integration, notifications | ✅ |
| `bizclaw-runtime` | Process adapters | ✅ |
| `bizclaw-platform` | Multi-tenant admin platform, JWT, audit log | ✅ |

### 📊 Stats

| Metric | Value |
|--------|-------|
| **Language** | 100% Rust |
| **Crates** | 14 |
| **Lines of Code** | ~32781 |
| **Tests** | 113 passing |
| **Providers** | 15 built-in + custom endpoint |
| **Channels** | 9 types |
| **Tools** | 13 native + MCP (unlimited) |
| **Gallery** | 51 business agent templates |
| **Dashboard** | 11 pages, bilingual (VI/EN) |
| **Binary Size** | bizclaw 12M, platform 7.2M |
| **Last Updated** | 2026-02-24 (eaec8d8) |

---

## 🇬🇧 English

### What is BizClaw?

BizClaw is a **self-hosted AI Agent platform** built entirely in Rust. Run AI agents on your own infrastructure — no cloud lock-in, no data leaving your servers.

### Key Features

- **🔌 15 Providers** — OpenAI, Anthropic, Gemini, DeepSeek, Groq, OpenRouter, Together, MiniMax, xAI, Mistral, Ollama, llama.cpp, Brain, CLIProxy, vLLM
- **💬 9 Channels** — CLI, Telegram, Discord, Email, Webhook, WhatsApp, Zalo
- **🛠️ 13 Tools** — Shell, File, Edit, Glob, Grep, Web Search, HTTP, Config, Execute Code (9 langs), Plan Mode, Group Summarizer, Calendar, Doc Reader, Memory Search, Session Context
- **🔗 MCP Support** — Connect any MCP server for unlimited tool extensions
- **🏢 Multi-Tenant Platform** — Admin dashboard, JWT auth, per-tenant isolated DB
- **🌐 Web Dashboard** — 11-page bilingual UI (Vietnamese/English), real-time WebSocket chat
- **🤖 51 Agent Templates** — Pre-built agents for HR, Sales, Finance, Ops, Legal, Marketing, IT
- **👥 Group Chat** — Multi-agent collaboration with mixed providers
- **🧠 3-Tier Memory** — Brain workspace + daily auto-compaction + FTS5 search
- **📚 Knowledge RAG** — Upload documents for retrieval-augmented generation
- **⏰ Scheduler** — Automated tasks with agent integration
- **🔒 Security** — AES-256, command allowlists, HMAC-SHA256, JWT + bcrypt

### Quick Start

```bash
git clone https://github.com/nguyenduchoai/bizclaw.git
cd bizclaw && cargo build --release
./target/release/bizclaw init
./target/release/bizclaw serve
# Open http://localhost:3579 for dashboard
```

### Deployment

BizClaw is deployed at [bizclaw.vn](https://bizclaw.vn):
- Admin Platform: `bizclaw.vn/admin/`
- Demo Tenant: `demo.bizclaw.vn`
- Sales Tenant: `sales.bizclaw.vn`

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

**BizClaw** v0.2.0 — *AI nhanh, mọi nơi. / Fast AI, everywhere.*
