FROM node:22-alpine AS web
ARG BUILD_SHA=dev
ENV VITE_BUILD_SHA=${BUILD_SHA}
WORKDIR /build
COPY package.json package-lock.json* tsconfig.json vite.config.ts ./
COPY frontend ./frontend
RUN npm ci --no-audit --no-fund && npm run build

FROM rust:1-alpine AS server
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

FROM alpine:3.22
ARG BUILD_SHA=dev
RUN apk add --no-cache ca-certificates && addgroup -S receipts && adduser -S -G receipts receipts \
    && mkdir /data && chown receipts:receipts /data
WORKDIR /app
COPY --from=server /build/target/release/telemetry-export-receipts /usr/local/bin/telemetry-export-receipts
COPY --from=web /build/dist ./dist
USER receipts
ENV PORT=8080 DATABASE_URL=sqlite:///data/receipts.db?mode=rwc TER_BUILD_SHA=${BUILD_SHA}
EXPOSE 8080
VOLUME ["/data"]
ENTRYPOINT ["telemetry-export-receipts"]
