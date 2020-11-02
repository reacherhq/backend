[![Actions Status](https://github.com/reacherhq/backend/workflows/pr/badge.svg)](https://github.com/reacherhq/backend/actions)
![GitHub](https://img.shields.io/github/license/reacherhq/backend.svg)
[![Github Sponsor](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&link=https://github.com/sponsors/amaurymartiny)](https://github.com/sponsors/amaurymartiny)

# Reacher Backend

This repo holds the backend for [Reacher](https://reacher.email). The backend is a HTTP server with the following components:
- [`check-if-email-exists`](https://github.com/amaurymartiny/check-if-email-exists), which performs the core email verification logic,
- [`warp`](https://github.com/seanmonstar/warp) web framework.

## Documentation: https://reacher.email/docs

## Get Started

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/reacherhq/backend)

To run the server locally on your machine, just clone the repository and run:

```bash
cargo run
```

The server will then be listening on `http://127.0.0.1:8080`.

These are the environment variables used to configure the HTTP server:

| Env Var          | Required? | Description                                                        | Default            |
| ---------------- | --------- | ------------------------------------------------------------------ | ------------------ |
| `RCH_FROM_EMAIL` | No        | The email to use in the `MAIL FROM:` SMTP command.                 | `user@example.org` |
| `RCH_HTTP_HOST`  | No        | The host name to bind the HTTP server to.                          | `127.0.0.1`        |
| `RCH_PROXY_HOST` | No        | Use the specified SOCKS5 proxy host to perform email verification. | not defined        |
| `RCH_PROXY_PORT` | No        | Use the specified SOCKS5 proxy port to perform email verification. | not defined        |
| `RCH_SENTRY_DSN` | No        | [Sentry](https://sentry.io) DSN used for bug reports.              | not defined        |

## Sponsor Me!

If you would like a high free tier to test Reacher, consider [sponsoring me](https://github.com/sponsors/amaurymartiny/)! You'll get 8000 free email verifications every month, and a this contribution would mean A WHOLE LOT to me.
