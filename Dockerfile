# syntax=docker/dockerfile:1@sha256:572cebbc2b15e6f84c077742952674889cb63cd5e9b80abdffcbe587b6f19183

# Packages a prebuilt static musl binary from dist/<arch>/sysslor (see ci.yml).
FROM scratch
ARG TARGETARCH
LABEL org.opencontainers.image.source="https://github.com/sebastianljunggren/sysslor" \
      org.opencontainers.image.licenses="MIT OR Apache-2.0"
# Artifact downloads don't keep the executable bit.
COPY --chmod=755 dist/${TARGETARCH}/sysslor /sysslor
ENV SYSSLOR_DATABASE_URL=sqlite:///data/sysslor.db \
    SYSSLOR_BIND_ADDR=0.0.0.0:8080
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/sysslor"]
