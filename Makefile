.PHONY: build test clippy fmt release docker clean help

# ── Default ────────────────────────────────────────────────────
help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

# ── Development ────────────────────────────────────────────────
build: ## Build all crates (debug)
	cargo build --workspace

test: ## Run all tests
	cargo test --workspace

clippy: ## Run clippy with deny warnings
	cargo clippy --workspace -- -D warnings

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check formatting
	cargo fmt --all -- --check

check: clippy test fmt-check ## Run all checks (clippy + test + fmt)

# ── Release ────────────────────────────────────────────────────
release: ## Build optimized release binary
	cargo build --release

# ── Docker ─────────────────────────────────────────────────────
docker: release ## Build Docker image (local arch only)
	@echo "Building local Docker image..."
	mkdir -p docker-bin/linux/amd64
	cp target/release/bizclaw docker-bin/linux/amd64/bizclaw
	docker build -t bizclaw:local .
	rm -rf docker-bin

docker-compose: ## Run via docker compose
	docker compose up -d

docker-down: ## Stop docker compose
	docker compose down

# ── Cleanup ────────────────────────────────────────────────────
clean: ## Clean build artifacts
	cargo clean
	rm -rf docker-bin dist
