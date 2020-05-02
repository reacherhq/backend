FROM alpine:latest

WORKDIR /reacher

ENV REACHER_VERSION 0.1.1

# Install needed libraries
RUN apk update && \
	apk add --no-cache openssl tor wget && \
	rm -rf /var/cache/apk/*

# Download the binary from Github Releases
RUN wget https://github.com/reacherhq/backend/releases/download/v${REACHER_VERSION}/reacher-v${REACHER_VERSION}-x86_64-unknown-linux-musl.tar.gz \
	&& tar -xvzf reacher-v${REACHER_VERSION}-x86_64-unknown-linux-musl.tar.gz

COPY ./scripts/docker_entrypoint.sh .

CMD ["./docker_entrypoint.sh"]
