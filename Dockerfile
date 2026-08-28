FROM node:22-alpine AS web
WORKDIR /build
COPY package.json package-lock.json* tsconfig.json vite.config.ts ./
COPY frontend ./frontend
RUN npm install --no-audit --no-fund && npm run build

FROM rust:1.89-alpine AS server
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release --locked

FROM alpine:3.22
RUN apk add --no-cache ca-certificates && addgroup -S receipts && adduser -S -G receipts receipts
WORKDIR /app
COPY --from=server /build/target/release/telemetry-export-receipts /usr/local/bin/telemetry-export-receipts
COPY --from=web /build/dist ./dist
RUN mkdir /app/data && chown receipts:receipts /app/data
USER receipts
ENV PORT=8080 DATABASE_URL=sqlite://data/receipts.db?mode=rwc TER_APP_ENV=production
EXPOSE 8080
VOLUME ["/app/data"]
ENTRYPOINT ["telemetry-export-receipts"]
