# Docker Image Build Troubleshooting

## Issue: Missing Docker Image on GHCR

If you don't see the Docker image at `ghcr.io/ngtrthanh/bizclaw`, follow these steps:

### 1. Check Workflow Status

Visit: https://github.com/ngtrthanh/bizclaw/actions

Look for the release workflow for your tag (e.g., v0.3.2):
- ✅ Green checkmark = Success
- ❌ Red X = Failed
- 🟡 Yellow circle = Running

### 2. Check Workflow Logs

Click on the workflow run → Click on "Build & Push Docker Image" job

Common issues:
- **"permission denied"** - GHCR permissions not configured
- **"artifact not found"** - Linux build failed
- **"push failed"** - Package visibility or permissions issue

### 3. Enable GHCR Package Permissions

#### Option A: Via GitHub Settings (Recommended)

1. Go to: https://github.com/ngtrthanh/bizclaw/settings/actions
2. Scroll to "Workflow permissions"
3. Select "Read and write permissions"
4. Check "Allow GitHub Actions to create and approve pull requests"
5. Click "Save"

#### Option B: Via Package Settings

1. Go to: https://github.com/users/ngtrthanh/packages/container/bizclaw/settings
2. Under "Danger Zone" → Change visibility to "Public"
3. Under "Manage Actions access" → Add repository with "Write" permission

### 4. Re-run Failed Workflow

If the workflow failed:

1. Go to: https://github.com/ngtrthanh/bizclaw/actions
2. Click on the failed workflow
3. Click "Re-run all jobs"

### 5. Manual Docker Build (Temporary Solution)

If you need the image immediately:

```bash
# Clone the repository
git clone https://github.com/ngtrthanh/bizclaw.git
cd bizclaw
git checkout v0.3.2

# Build locally (requires cross-compilation)
cargo install cross
cross build --release --target x86_64-unknown-linux-gnu

# Prepare Docker context
mkdir -p docker-bin/linux/amd64
cp target/x86_64-unknown-linux-gnu/release/bizclaw docker-bin/linux/amd64/

# Build Docker image
docker build --platform linux/amd64 --build-arg TARGETPLATFORM=linux/amd64 -t bizclaw:v0.3.2 .

# Tag for GHCR (optional)
docker tag bizclaw:v0.3.2 ghcr.io/ngtrthanh/bizclaw:v0.3.2
docker tag bizclaw:v0.3.2 ghcr.io/ngtrthanh/bizclaw:latest

# Push to GHCR (requires authentication)
echo $GITHUB_TOKEN | docker login ghcr.io -u ngtrthanh --password-stdin
docker push ghcr.io/ngtrthanh/bizclaw:v0.3.2
docker push ghcr.io/ngtrthanh/bizclaw:latest
```

### 6. Check Package Visibility

Visit: https://github.com/ngtrthanh?tab=packages

You should see "bizclaw" package listed. If not:
- The workflow hasn't completed yet
- The workflow failed
- Permissions are not configured

### 7. Verify Image Exists

Once the workflow succeeds:

```bash
# Pull the image
docker pull ghcr.io/ngtrthanh/bizclaw:v0.3.2

# Check it works
docker run --rm ghcr.io/ngtrthanh/bizclaw:v0.3.2 --version
```

## Common Error Messages

### "Error: failed to push to ghcr.io"

**Cause:** No write permission to GHCR

**Fix:**
1. Go to repository Settings → Actions → Workflow permissions
2. Enable "Read and write permissions"
3. Re-run the workflow

### "Error: artifact 'bizclaw-linux-amd64' not found"

**Cause:** Linux build job failed

**Fix:**
1. Check the "Build Linux" job logs
2. Fix any compilation errors
3. Re-run the workflow

### "Error: denied: permission_denied"

**Cause:** Package doesn't exist or no write access

**Fix:**
1. Create the package manually by pushing once:
   ```bash
   docker tag bizclaw:local ghcr.io/ngtrthanh/bizclaw:test
   docker push ghcr.io/ngtrthanh/bizclaw:test
   ```
2. Then configure package permissions
3. Re-run the workflow

## Alternative: Use Docker Hub

If GHCR continues to have issues, you can use Docker Hub instead:

### Update Workflow

Edit `.github/workflows/release.yml`:

```yaml
- name: Login to Docker Hub
  uses: docker/login-action@v3
  with:
    username: ${{ secrets.DOCKERHUB_USERNAME }}
    password: ${{ secrets.DOCKERHUB_TOKEN }}

- name: Build and push multi-arch image
  uses: docker/build-push-action@v6
  with:
    context: .
    file: Dockerfile
    platforms: linux/amd64,linux/arm64,linux/arm/v7,linux/arm/v6
    push: true
    tags: |
      ngtrthanh/bizclaw:latest
      ngtrthanh/bizclaw:${{ steps.version.outputs.VERSION }}
```

### Add Secrets

1. Go to: https://github.com/ngtrthanh/bizclaw/settings/secrets/actions
2. Add `DOCKERHUB_USERNAME` and `DOCKERHUB_TOKEN`

## Verification Checklist

- [ ] Workflow completed successfully (green checkmark)
- [ ] "Build & Push Docker Image" job succeeded
- [ ] Package visible at https://github.com/ngtrthanh?tab=packages
- [ ] Image pulls successfully: `docker pull ghcr.io/ngtrthanh/bizclaw:latest`
- [ ] Image runs: `docker run --rm ghcr.io/ngtrthanh/bizclaw:latest --version`

## Getting Help

If issues persist:

1. **Check workflow logs** for specific error messages
2. **Verify permissions** in repository settings
3. **Try manual build** to isolate the issue
4. **Check GitHub status** at https://www.githubstatus.com/

## Quick Fix Summary

Most common fix:
1. Go to: https://github.com/ngtrthanh/bizclaw/settings/actions
2. Enable "Read and write permissions"
3. Re-run the failed workflow

---

**Last Updated**: February 25, 2025
