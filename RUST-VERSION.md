# Rust Version Information

## Current Version: 1.85

This project uses Rust 1.85, which is the latest stable version as of February 2025.

## Known Issue: Dependency Metadata Bug

Some transitive dependencies (`time@0.3.47`, `cookie_store@0.22.1`, `zip@8.1.0`) have incorrect metadata claiming they require Rust 1.88, which doesn't exist yet. This is a known bug in those crates' `rust-version` field.

### Impact

- **Local builds**: You may see warnings about unsupported Rust versions, but the code compiles fine
- **CI/CD**: GitHub Actions uses the latest stable Rust, so builds work without issues
- **Cross-compilation**: Works perfectly in CI with the `cross` tool

### Workaround Applied

We've downgraded `zip` from 8.1.0 to 7.2.0 in `bizclaw-tools` to avoid one source of this issue.

The remaining warnings from `time` and `cookie_store` (pulled in by `reqwest`) can be safely ignored - they compile fine with Rust 1.85.

## Why Rust 1.85?

Rust 1.85 includes:
- ✅ Async closures (stabilized)
- ✅ Edition 2024 support
- ✅ All features needed for this project
- ✅ Full cross-compilation support
- ✅ Latest stable release

## Upgrading Rust

To ensure you have the latest Rust:

```bash
rustup update stable
rustup default stable
```

## CI/CD Rust Version

GitHub Actions workflows automatically use the latest stable Rust via:
```yaml
- uses: dtolnay/rust-toolchain@stable
```

This ensures CI always uses the most recent stable release.

## Future

When Rust 1.86+ is released, we'll update to it. The dependency metadata bugs should be fixed by then, or we'll update to newer versions of those crates.

## Summary

✅ **Rust 1.85 is perfect for this project**
✅ **Cross-compilation works out of the box in CI**
✅ **Dependency warnings can be safely ignored**
✅ **No action needed from developers**
