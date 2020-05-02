FROM alpine:latest

WORKDIR /reacher

ENV REACHER_VERSION 0.1.1

# Install needed libraries
RUN apk update && \
	apk add --no-cache openssl tor wget && \
	rm -rf /var/cache/apk/*

COPY ./reacher .
COPY ./scripts/docker_entrypoint.sh .

CMD ["./docker_entrypoint.sh"]
