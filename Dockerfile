# From https://shaneutt.com/blog/rust-fast-small-docker-image-builds/

# ------------------------------------------------------------------------------
# Cargo Build Stage
# ------------------------------------------------------------------------------

FROM messense/rust-musl-cross:x86_64-musl as cargo-build

WORKDIR /usr/src/reacher

RUN rm -f target/x86_64-unknown-linux-musl/release/deps/reacher*

COPY . .

ENV SQLX_OFFLINE=true

RUN cargo build --release --target=x86_64-unknown-linux-musl

# ------------------------------------------------------------------------------
# Final Stage
# ------------------------------------------------------------------------------

FROM alpine:latest

RUN addgroup -g 1000 reacher

RUN adduser -D -s /bin/sh -u 1000 -G reacher reacher

WORKDIR /home/reacher/bin/

COPY --from=cargo-build /usr/src/reacher/target/x86_64-unknown-linux-musl/release/heroku .

RUN chown reacher:reacher heroku

USER reacher

ENV RUST_LOG=reacher=info
ENV RCH_HTTP_HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080

CMD ["./heroku"]
