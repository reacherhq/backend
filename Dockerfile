FROM alpine:latest

WORKDIR /reacher

# Install needed libraries
RUN apk update && \
	apk add --no-cache openssl && \
	rm -rf /var/cache/apk/*
# Install Tor
RUN apk add tor
# Run Tor on port 9050
RUN tor &

# Copy the binary
COPY target/x86_64-unknown-linux-musl/release/reacher .

CMD ["./reacher"]
