FROM alpine:latest

WORKDIR /reacher

ENV RCH_HTTP_HOST 0.0.0.0
ENV RCH_PROXY_HOST 127.0.0.1
ENV RCH_PROXY_PORT 9050
ENV RUST_LOG debug

# Install needed libraries
RUN apk update && \
	apk add --no-cache openssl tor && \
	rm -rf /var/cache/apk/*

# Assumes a `./reacher` binary in the root folder. This ./reacher binary is
# built on CI, but you can also copy it from ./target/{debug,release}.
COPY ./reacher .
COPY ./scripts/docker_entrypoint.sh .

CMD ["./docker_entrypoint.sh"]
