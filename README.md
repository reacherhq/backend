[![Actions Status](https://github.com/reacherhq/backend/workflows/pr/badge.svg)](https://github.com/reacherhq/backend/actions)
![GitHub](https://img.shields.io/github/license/reacherhq/backend.svg)

# Reacher Backend

This repo holds the backend for [Reacher](https://reacherhq.github.io/). The backend is a HTTP server around the Rust library [`check-if-email-exists`](https://github.com/amaurymartiny/check-if-email-exists), which performs the core email verification logic.

The OpenAPIv3 specification of this backend can be seen on [StopLight](https://stoplight.io/p/docs/gh/reacherhq/backend).

## See also

-   [`check-if-email-exists`](https://github.com/amaurymartiny/check-if-email-exists): Rust library to check if an email address exists without sending any email.
