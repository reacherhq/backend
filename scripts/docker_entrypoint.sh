#!/bin/sh
set -e

# Run Tor in background, exposes port 9050
tor &

# Run Reacher with logs
RUST_LOG=debug ./reacher
