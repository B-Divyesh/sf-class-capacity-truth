FROM node:22-alpine AS web
WORKDIR /source
COPY package.json package-lock.json ./
RUN npm ci
COPY index.html tsconfig.json tsconfig.app.json tsconfig.node.json vite.config.ts ./
COPY public ./public
COPY src ./src
RUN npm run build:web

FROM rust:1-alpine AS api
ARG BUILD_SHA=dev
ARG GIT_SHA=dev
ARG SOURCE_COMMIT=dev
ENV BUILD_SHA=${BUILD_SHA}
RUN apk add --no-cache musl-dev
WORKDIR /source
COPY services/api/Cargo.toml services/api/Cargo.lock* ./services/api/
COPY services/api/migrations ./services/api/migrations
COPY services/api/src ./services/api/src
RUN cargo build --release --manifest-path services/api/Cargo.toml

FROM alpine:3.22 AS runtime
ARG BUILD_SHA=dev
ENV BUILD_SHA=${BUILD_SHA}
RUN addgroup -S app && adduser -S -G app -h /app app \
    && mkdir -p /app/dist /data \
    && chown -R app:app /app /data
COPY --from=web --chown=app:app /source/dist /app/dist
COPY --from=api --chown=app:app /source/services/api/target/release/class-capacity-truth-api /usr/local/bin/class-capacity-truth-api
USER app
WORKDIR /app
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 CMD wget -q -O- http://127.0.0.1:${PORT:-8080}/health || exit 1
ENTRYPOINT ["class-capacity-truth-api"]
