FROM alpine:latest

# Install needed libraries
RUN apk update && \
	apk add --no-cache tor && \
	rm -rf /var/cache/apk/*

RUN echo "Log notice stdout" >> /etc/torrc
RUN echo "SocksPort 0.0.0.0:9050" >> /etc/torrc

EXPOSE 9050

CMD ["tor", "-f", "/etc/torrc"]
