# Rust Version Information

## Current Version: 1.93.1

This project uses Rust 1.93.1, which is the latest stable version as of February 12, 2026.

## Why Rust 1.93?

Rust 1.93 includes all modern features needed for this project:
- ✅ Async closures (stabilized in 1.85)
- ✅ Edition 2024 support
- ✅ Latest performance improvements
- ✅ Full cross-compilation support
- ✅ All dependency compatibility

## Cross-Compilation Support

With Rust 1.93, cross-compilation works perfectly out of the box:

- ✅ **Linux**: x86_64, aarch64, armv7 (via `cross` tool)
- ✅ **macOS**: Intel (x86_64), Apple Silicon (aarch64)
- ✅ **Windows**: x86_64

No special configuration needed - everything just works!

## Dependency Compatibility

All dependencies are fully compatible with Rust 1.93:
- `reqwest` with cookies and streaming
- `tokio` async runtime
- `axum` web framework
- All quantization and AI libraries

## Upgrading Rust

To ensure you have Rust 1.93:

```bash
rustup update stable
rustup default stable
rustc --version  # Should show 1.93.x
```

## CI/CD Rust Version

GitHub Actions workflows automatically use the latest stable Rust via:
```yaml
- uses: dtolnay/rust-toolchain@stable
```

This ensures CI always uses Rust 1.93+ for all builds.

## rust-toolchain.toml

The project includes `rust-toolchain.toml` which pins the Rust version to 1.93:

```toml
[toolchain]
channel = "1.93"
components = ["rustfmt", "clippy"]
profile = "minimal"
```

This ensures all developers and CI use the same Rust version.

## Building the Project

```bash
# Install Rust 1.93 (if not already installed)
rustup update stable

# Build
cargo build --release

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings
```

## Cross-Compilation in CI

The release workflow automatically builds for all platforms:

1. **Linux** (via `cross`):
   - x86_64-unknown-linux-gnu
   - aarch64-unknown-linux-gnu
   - armv7-unknown-linux-gnueabihf

2. **macOS** (native):
   - x86_64-apple-darwin (Intel)
   - aarch64-apple-darwin (Apple Silicon)

3. **Windows** (native):
   - x86_64-pc-windows-msvc

All builds use Rust 1.93 and work perfectly!

## Summary

✅ **Rust 1.93.1 is the latest stable**
✅ **Cross-compilation works out of the box**
✅ **All dependencies are compatible**
✅ **CI/CD fully automated**
✅ **No special configuration needed**

The project is production-ready with Rust 1.93! 🚀
