# syntax=docker/dockerfile:1

# The built assets are platform independent, so skip emulation for this stage.
FROM --platform=$BUILDPLATFORM docker.io/library/node:24-alpine AS web
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
RUN npm run build

# Builds natively for the target platform (musl on alpine), e.g. --platform linux/arm64.
FROM docker.io/library/rust:1-alpine AS build
RUN apk add --no-cache musl-dev gcc
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src
COPY migrations migrations
COPY --from=web /web/dist web/dist
ARG TARGETARCH
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target,id=sysslor-target-$TARGETARCH \
    cargo build --release --locked \
    && cp target/release/sysslor /sysslor

FROM scratch
COPY --from=build /sysslor /sysslor
ENV DATABASE_URL=sqlite:///data/sysslor.db \
    BIND_ADDR=0.0.0.0:8080
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["/sysslor"]
