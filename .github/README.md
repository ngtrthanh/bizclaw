# CI/CD Documentation

This directory contains GitHub Actions workflows for continuous integration and deployment.

## Workflows

### CI (`ci.yml`)

Runs on every push and pull request to `master`:

- **Test Suite**: Runs all tests with `cargo test`
- **Rustfmt**: Checks code formatting
- **Clippy**: Lints code with `-D warnings`
- **Build Check**: Builds on Linux, macOS, and Windows

### Release (`release.yml`)

Triggers on version tags (e.g., `v0.1.0`):

1. **Multi-platform Builds**:
   - Linux: x86_64, aarch64, armv7
   - macOS: x86_64 (Intel), aarch64 (Apple Silicon)
   - Windows: x86_64

2. **GitHub Release**: Creates release with all binaries

3. **Docker Image**: Builds and pushes multi-arch image to GHCR

## Creating a Release

1. Update version in `Cargo.toml`
2. Commit changes: `git commit -am "chore: bump version to X.Y.Z"`
3. Create and push tag:
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
4. GitHub Actions will automatically:
   - Build binaries for all platforms
   - Create GitHub release
   - Build and push Docker images

## Docker Images

Images are published to GitHub Container Registry:

```bash
# Pull latest
docker pull ghcr.io/ngtrthanh/bizclaw:latest

# Pull specific version
docker pull ghcr.io/ngtrthanh/bizclaw:v0.1.0

# Run
docker run -p 8080:8080 ghcr.io/ngtrthanh/bizclaw:latest
```

### Supported Architectures

- `linux/amd64` (x86_64)
- `linux/arm64` (aarch64)
- `linux/arm/v7` (armv7)

## Local Development

### Using Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f bizclaw

# Stop services
docker-compose down
```

### Building Locally

```bash
# Build
cargo build --release

# Run tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace --all-targets -- -D warnings
```

## Caching

All workflows use GitHub Actions cache to speed up builds:
- Cargo registry
- Cargo git dependencies
- Build artifacts

Cache keys are based on:
- OS
- Target architecture
- `Cargo.lock` hash

## Secrets Required

For releases, ensure these secrets are set in repository settings:

- `GITHUB_TOKEN` (automatically provided by GitHub Actions)

No additional secrets needed for basic CI/CD!

## Troubleshooting

### Build Failures

1. Check if `Cargo.lock` is up to date
2. Verify all dependencies are compatible
3. Check clippy warnings with `-D warnings`

### Docker Build Issues

1. Ensure binaries are built before Docker step
2. Check `docker-bin/` directory structure
3. Verify TARGETPLATFORM matches artifact paths

### Release Not Triggering

1. Ensure tag starts with `v` (e.g., `v0.1.0`)
2. Check tag was pushed: `git push origin --tags`
3. Verify workflow permissions in repository settings
