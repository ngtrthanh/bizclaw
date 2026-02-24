# Docker Signal Handling Fix

## Problem

When running BizClaw in Docker, you may encounter this error:

```
thread 'main' panicked at /cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tokio-1.49.0/src/signal/unix.rs:72:53:
failed to create UnixStream: Os { code: 13, kind: PermissionDenied, message: "Permission denied" }
```

This occurs because Tokio's Unix signal handling (`tokio::signal::ctrl_c()`) requires creating Unix domain sockets, which needs specific permissions in Docker containers.

## Solutions

### Solution 1: Use `init: true` (Recommended)

The `docker-compose.yml` has been updated to include `init: true`:

```yaml
services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:latest
    init: true  # ← This fixes signal handling
    cap_add:
      - SYS_PTRACE
    volumes:
      - /tmp  # ← Ensure /tmp is writable
```

**What it does:**
- Uses tini as PID 1 to properly handle signals
- Forwards SIGTERM/SIGINT to the application
- Cleans up zombie processes

### Solution 2: Add Capabilities

If `init: true` doesn't work, add the `SYS_PTRACE` capability:

```yaml
services:
  bizclaw:
    cap_add:
      - SYS_PTRACE
```

### Solution 3: Run as Root (Not Recommended)

Only use this for testing:

```yaml
services:
  bizclaw:
    user: root
```

**Warning:** Running as root is a security risk. Use only for debugging.

### Solution 4: Use Docker Run

If using `docker run` instead of docker-compose:

```bash
# With init
docker run --init ghcr.io/ngtrthanh/bizclaw:latest

# With capabilities
docker run --cap-add=SYS_PTRACE ghcr.io/ngtrthanh/bizclaw:latest

# With both (recommended)
docker run --init --cap-add=SYS_PTRACE ghcr.io/ngtrthanh/bizclaw:latest
```

## Why This Happens

### Technical Details

1. **Tokio Signal Handling**
   - Tokio uses Unix domain sockets for signal handling
   - Creates sockets in `/tmp` or `$TMPDIR`
   - Requires write permissions and proper file descriptors

2. **Docker Security**
   - Containers run with restricted capabilities by default
   - Non-root users have limited permissions
   - Signal handling requires `SYS_PTRACE` capability

3. **PID 1 Problem**
   - Docker containers run the main process as PID 1
   - PID 1 has special signal handling behavior
   - Signals may not be properly forwarded without init

## Verification

### Test Signal Handling

```bash
# Start container
docker-compose up -d

# Check logs
docker-compose logs -f

# Send SIGTERM (should gracefully shutdown)
docker-compose stop

# Should see: "Channels stopped" or similar message
```

### Check Permissions

```bash
# Enter container
docker exec -it bizclaw sh

# Check /tmp permissions
ls -la /tmp

# Should show: drwxrwxrwt (sticky bit set)

# Check user
whoami
# Should show: bizclaw

# Check capabilities
cat /proc/1/status | grep Cap
```

## Alternative: Disable Signal Handling

If you don't need graceful shutdown, you can modify the code to skip signal handling:

```rust
// Instead of:
tokio::signal::ctrl_c().await?;

// Use:
tokio::time::sleep(tokio::time::Duration::MAX).await;
```

**Note:** This is not recommended as it prevents graceful shutdown.

## Docker Compose Full Example

```yaml
version: '3.8'

services:
  bizclaw:
    image: ghcr.io/ngtrthanh/bizclaw:latest
    container_name: bizclaw
    restart: unless-stopped
    
    # Signal handling fixes
    init: true
    cap_add:
      - SYS_PTRACE
    
    # Ensure /tmp is writable
    tmpfs:
      - /tmp:exec,mode=1777
    
    ports:
      - "8080:8080"
    
    volumes:
      - ./config:/home/bizclaw/.bizclaw
      - ./data:/home/bizclaw/data
    
    environment:
      - RUST_LOG=info
      - TMPDIR=/tmp
    
    healthcheck:
      test: ["CMD", "/usr/local/bin/bizclaw", "--version"]
      interval: 30s
      timeout: 3s
      retries: 3
```

## Kubernetes

For Kubernetes deployments:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: bizclaw
spec:
  containers:
  - name: bizclaw
    image: ghcr.io/ngtrthanh/bizclaw:latest
    securityContext:
      capabilities:
        add:
        - SYS_PTRACE
    volumeMounts:
    - name: tmp
      mountPath: /tmp
  volumes:
  - name: tmp
    emptyDir: {}
```

## Troubleshooting

### Still Getting Permission Denied?

1. **Check Docker version**
   ```bash
   docker --version
   # Should be 20.10+ for init support
   ```

2. **Check SELinux/AppArmor**
   ```bash
   # Temporarily disable SELinux
   sudo setenforce 0
   
   # Or add security-opt
   docker run --security-opt label=disable ...
   ```

3. **Check tmpfs mount**
   ```bash
   docker exec bizclaw mount | grep tmp
   # Should show tmpfs mounted on /tmp
   ```

4. **Rebuild image**
   ```bash
   docker-compose build --no-cache
   docker-compose up -d
   ```

### Logs Show "Permission denied" on /tmp

```bash
# Fix /tmp permissions in Dockerfile
RUN chmod 1777 /tmp
```

### Container Exits Immediately

```bash
# Check logs
docker-compose logs bizclaw

# Run interactively
docker run -it --init ghcr.io/ngtrthanh/bizclaw:latest sh

# Test signal handling
kill -TERM 1
```

## Best Practices

1. **Always use `init: true`** in docker-compose
2. **Add `SYS_PTRACE` capability** for signal handling
3. **Mount /tmp as tmpfs** with exec permissions
4. **Set TMPDIR environment variable** explicitly
5. **Test graceful shutdown** before production

## References

- [Tokio Signal Handling](https://docs.rs/tokio/latest/tokio/signal/)
- [Docker Init Process](https://docs.docker.com/engine/reference/run/#specify-an-init-process)
- [Docker Capabilities](https://docs.docker.com/engine/reference/run/#runtime-privilege-and-linux-capabilities)
- [Tini - A tiny init](https://github.com/krallin/tini)

---

**Last Updated**: February 24, 2025
