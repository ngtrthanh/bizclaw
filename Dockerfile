# Multi-stage Dockerfile for BizClaw
# Supports multi-arch: linux/amd64, linux/arm64, linux/arm/v7

FROM debian:bookworm-slim

ARG TARGETPLATFORM

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy pre-built binary based on platform
COPY docker-bin/${TARGETPLATFORM}/bizclaw /usr/local/bin/bizclaw

# Create non-root user
RUN useradd -m -u 1000 bizclaw && \
    mkdir -p /home/bizclaw/.bizclaw && \
    chown -R bizclaw:bizclaw /home/bizclaw

USER bizclaw
WORKDIR /home/bizclaw

# Expose default ports
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/bizclaw", "--version"]

ENTRYPOINT ["/usr/local/bin/bizclaw"]
CMD ["--help"]
