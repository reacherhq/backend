FROM alpine:latest

WORKDIR /reacher

ENV REACHER_VERSION 0.4.1

# Install needed libraries
RUN apk update && \
	apk add --no-cache openssl wget && \
	rm -rf /var/cache/apk/*
# Install Tor
RUN apk add tor
# Run Tor on port 9050
RUN tor &

# Download the binary from Github Releases
RUN wget https://github.com/reacherhq/microservices-releases/releases/download/v${REACHER_VERSION}/reacher-v${REACHER_VERSION}-x86_64-unknown-linux-musl.tar.gz \
	&& tar -xvzf reacher-v${REACHER_VERSION}-x86_64-unknown-linux-musl.tar.gz

CMD ["./reacher"]
