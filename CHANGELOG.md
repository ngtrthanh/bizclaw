# Changelog

All notable changes to BizClaw will be documented in this file.

## [0.1.0] — 2026-02-22

### 🧠 Brain Engine — Inference Backend Modernization
- **K-quant support**: Added dequantization kernels for Q2_K, Q3_K, Q4_K, Q5_K, Q6_K formats
- **Strict validation**: Models with unsupported tensor types are now rejected at load time (no more silent zero-fill)
- **Grammar-constrained JSON**: `generate_json()` now uses `JsonGrammar.apply_mask()` for syntactically valid JSON output
- **Feature flags**: Added `rust-native-backend` (default) and `llama-backend` feature flags

### 🔐 Security
- **AES-256-GCM**: Upgraded secret encryption from ECB to GCM (authenticated encryption with nonce)
- **Auto-migration**: Legacy ECB-encrypted `secrets.enc` files are automatically re-encrypted on first load
- **Tamper detection**: GCM provides integrity verification — corrupted ciphertext is rejected

### 🚀 CI/CD Pipeline
- **CI workflow** (`ci.yml`): Build + test + clippy (deny warnings) + format check on push to `main` / PR
- **Release workflow** (`release.yml`): Tag-triggered multi-arch release:
  - Linux: x86_64, aarch64, armv7, armv6 (via `cross`)
  - macOS: Intel (x86_64) + Apple Silicon (aarch64) (native runners)
  - Windows: x86_64 (native runner)
- **Docker**: Multi-arch image build + push to `ghcr.io` (linux/amd64, linux/arm64, linux/arm/v7)
- **Cargo caching**: All CI jobs use cargo registry + target caching

### 📦 Distribution
- **Dockerfile**: Multi-arch, non-root runtime, `debian:bookworm-slim` base, health check
- **docker-compose.yml**: Config + model volume mounts, port 3000, health check
- **install.sh**: Detects OS (Linux/macOS) and arch, downloads correct binary from GitHub Releases

### 🛠 Developer Experience
- **Makefile**: `build`, `test`, `clippy`, `fmt`, `check`, `release`, `docker`, `clean`
- **.devcontainer**: Rust dev container with rust-analyzer, clippy, LLDB debugger

### 📖 Documentation
- **README audit**: Fixed overclaims (Flash Attention → Online-Softmax, crate count 12→11, test count 45→66)
- **Quantization**: Updated all quant tables to reflect K-quant support
- **Encryption**: Updated from "AES-256" to "AES-256-GCM (authenticated)"
- **Distribution section**: Added install.sh, Docker, and binary download instructions
- **Developer section**: Added Makefile commands and devcontainer reference

### 🐛 Bug Fixes
- Fixed missing `agent` field in gateway test state constructor (pre-existing compile error)
- Removed unused imports in security and brain crates
