# syntax=docker/dockerfile:1.7
# arena-of-ideas: one image ships the whole game — the arena server (Hono +
# better-sqlite3) serving the built web client same-origin from STATIC_DIR.
# Deploy pattern mirrors the hub's void-mail image; two pinned divergences:
#
#   - base is bookworm-slim, not alpine: better-sqlite3 is a native module
#     with glibc prebuilds — alpine (musl) would need a compile toolchain.
#   - runtime executes TypeScript via tsx (as void-mail does); a tsc-compile
#     step is NOT an option here, because the kernel imports
#     registry/approved-units.json as an ES module, which plain node only
#     accepts with import attributes tsx adds on the fly.

# ---- Stage 1: full install + web client build ----
FROM node:22-bookworm-slim AS build
WORKDIR /app
# Dev deps include playwright; its browser download has no place in an image.
ENV PLAYWRIGHT_SKIP_BROWSER_DOWNLOAD=1
COPY package.json package-lock.json ./
COPY server/package.json server/
RUN npm ci --no-audit --no-fund
COPY tsconfig.json vite.config.ts ./
COPY src ./src
COPY web ./web
COPY registry ./registry
COPY examples ./examples
# vite → /app/dist (the repo-root outDir from vite.config.ts)
RUN npm run build

# ---- Stage 2: production dependencies only ----
FROM node:22-bookworm-slim AS deps
WORKDIR /app
COPY package.json package-lock.json ./
COPY server/package.json server/
RUN npm ci --omit=dev --no-audit --no-fund

# ---- Stage 3: runtime ----
FROM node:22-bookworm-slim AS runtime
WORKDIR /app

# tini = PID 1 reaper + signal forwarder (void-mail pattern); wget feeds the
# healthcheck — bookworm-slim ships neither.
RUN apt-get update \
    && apt-get install -y --no-install-recommends tini wget \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd -r arena && useradd -r -g arena arena

ENV NODE_ENV=production \
    PORT=8787 \
    DB_PATH=/data/arena.db \
    STATIC_DIR=/app/public

COPY --from=deps /app/node_modules ./node_modules
COPY package.json ./
COPY server/package.json ./server/package.json
COPY server/src ./server/src
COPY src ./src
COPY registry ./registry
COPY --from=build /app/dist ./public

# SQLite home. Named volumes copy this ownership on first use, so the non-root
# user can write; a bind mount would need the chown done host-side.
RUN mkdir -p /data && chown arena:arena /data

HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 \
    CMD wget -q -O - http://127.0.0.1:${PORT}/healthz || exit 1

USER arena
EXPOSE 8787
ENTRYPOINT ["/usr/bin/tini", "--"]
CMD ["node", "--import", "tsx/esm", "server/src/main.ts"]
