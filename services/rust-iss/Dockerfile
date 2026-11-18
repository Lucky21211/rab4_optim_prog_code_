FROM rust:1.92.0-bookworm AS build
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --locked --release
FROM build AS test
CMD ["cargo", "test", "--locked"]
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
COPY --from=build /app/target/release/rust_iss /usr/local/bin/rust_iss
USER app
ENV RUST_LOG=info
EXPOSE 3000
CMD ["rust_iss"]
