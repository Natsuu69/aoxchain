FROM rust:1.90-bookworm AS builder
WORKDIR /app
COPY . .
RUN cargo build -p aoxcmd --release

FROM debian:bookworm-slim
WORKDIR /opt/aoxchain
COPY --from=builder /app/target/release/aoxcmd /usr/local/bin/aoxcmd
EXPOSE 26656 8545
ENTRYPOINT ["aoxcmd"]
CMD ["node-bootstrap"]
