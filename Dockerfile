# ── Runtime image (uses pre-built binaries) ────────────────────
# For CI: binaries are placed in docker-bin/<platform>/bizclaw
# For local: build with --build-arg and provide binary
FROM debian:bookworm-slim

ARG TARGETPLATFORM

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/* && \
    useradd -m -s /bin/sh bizclaw

# Copy the pre-built binary for the target platform
COPY docker-bin/${TARGETPLATFORM}/bizclaw /usr/local/bin/bizclaw
RUN chmod +x /usr/local/bin/bizclaw

# Create data directories
RUN mkdir -p /home/bizclaw/.bizclaw/models && \
    chown -R bizclaw:bizclaw /home/bizclaw

USER bizclaw
WORKDIR /home/bizclaw

EXPOSE 3000

HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD ["/usr/local/bin/bizclaw", "serve", "--health-check"] || exit 1

ENTRYPOINT ["/usr/local/bin/bizclaw"]
CMD ["serve", "--host", "0.0.0.0", "--port", "3000"]
