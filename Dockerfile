# Supports multi-arch: linux/amd64, linux/arm64, linux/arm/v7, linux/arm/v6
# Compatible with: x86_64, Raspberry Pi 4 (arm64), Pi 3 (armv7), Pi Zero/1 (armv6)

# ARG must be declared before FROM so Buildx injects it correctly
# during multi-platform builds. Re-declaring after FROM is not needed
# here because it is only used in COPY, which runs before any RUN.
ARG TARGETPLATFORM

FROM debian:bookworm-slim

ARG TARGETPLATFORM

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy pre-built binary for the current target platform
COPY docker-bin/${TARGETPLATFORM}/bizclaw /usr/local/bin/bizclaw

# Create non-root user
RUN useradd -m -u 1000 bizclaw && \
    mkdir -p /home/bizclaw/.bizclaw && \
    chown -R bizclaw:bizclaw /home/bizclaw

USER bizclaw
WORKDIR /home/bizclaw

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD ["/usr/local/bin/bizclaw", "--version"]

ENTRYPOINT ["/usr/local/bin/bizclaw"]
CMD ["--help"]