FROM rust:1.98-bookworm AS builder
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/.cargo ./.cargo
COPY backend/src ./src
RUN cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /build/target/release/inviteflow-api /usr/local/bin/inviteflow-api
EXPOSE 8080
CMD ["inviteflow-api"]
