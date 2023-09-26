FROM rust:slim-bullseye as builder

WORKDIR /usr/src/ecd_token_resolver
COPY . .

RUN cargo build --release

FROM debian:bullseye-slim

RUN apt-get update && \
  apt-get install -y --no-install-recommends \
  chromium && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/ecd_token_resolver/target/release/ecd-token-resolver /usr/local/bin/ecd-token-resolver

CMD ["ecd-token-resolver"]

