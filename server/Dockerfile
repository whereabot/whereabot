FROM rust:1-bookworm AS builder
WORKDIR /app
# Build dependencies first with dummy sources so this layer caches.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && touch src/lib.rs \
    && cargo build --release && rm -rf src
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=builder /app/target/release/whereabouts-server /usr/local/bin/whereabouts-server
# Cloud Run sets PORT at runtime; the default matches the local port.
ENV PORT=8080
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/whereabouts-server"]
