[![Actions Status](https://github.com/reacherhq/backend/workflows/pr/badge.svg)](https://github.com/reacherhq/backend/actions)
[![Github Sponsor](https://img.shields.io/static/v1?label=Sponsor&message=%E2%9D%A4&logo=GitHub&link=https://github.com/sponsors/amaurym)](https://github.com/sponsors/amaurym)

<br /><br />

<p align="center"><img align="center" src="https://storage.googleapis.com/saasify-uploads-prod/696e287ad79f0e0352bc201b36d701849f7d55e7.svg" height="96" alt="reacher" /></p>
<h1 align="center">⚙️ Reacher Backend</h1>
<h4 align="center">Backend Server for Reacher Email Verification API: https://reacher.email.</h4>

<br /><br />

This repository holds the backend for [Reacher](https://reacher.email). The backend is a HTTP server with the following components:

-   [`check-if-email-exists`](https://github.com/reacherhq/check-if-email-exists), which performs the core email verification logic,
-   [`warp`](https://github.com/seanmonstar/warp) web framework.

## Get Started

There are 3 ways you can run this backend.

### 1. One-Click Deploy to Heroku

[![Deploy](https://www.herokucdn.com/deploy/button.svg)](https://heroku.com/deploy?template=https://github.com/reacherhq/backend)

After clicking on the button, just follow the instructions on screen.

### 2. Use Docker

The [Docker image](./Dockerfile) is hosted on Docker Hub: https://hub.docker.com/r/reacherhq/backend.

To run it, run the following command:

```bash
docker run -p 8080:8080 reacherhq/backend
```

You can then send a POST request with the following body to `http://localhost:8080/v0/check_email`:

```js
{
	"to_email": "someone@gmail.com",
	"from_email": "my@my-server.com", // optional, defaults to "user@example.org"
	"hello_name": "my-server.com",    // optional, defaults to "localhost"
	"proxy": {                        // optional, default is empty
		"host": "my-proxy.io",
		"port": "1080"
	}
}
```

### 3. Run locally

If you prefer to run the server locally on your machine, just clone the repository and run:

```bash
cargo run
```

The server will then be listening on `http://127.0.0.1:8080`.

### Configuration

These are the environment variables used to configure the HTTP server:

| Env Var              | Required? | Description                                                                                                       | Default            |
| -------------------- | --------- | ----------------------------------------------------------------------------------------------------------------- | ------------------ |
| `RCH_FROM_EMAIL`     | No        | The email to use in the `MAIL FROM:` SMTP command.                                                                | `user@example.org` |
| `RCH_HTTP_HOST`      | No        | The host name to bind the HTTP server to.                                                                         | `127.0.0.1`        |
| `PORT`               | No        | The port to bind the HTTP server to, populated by Heroku.                                                         | `8080`             |
| `RCH_SENTRY_DSN`     | No        | If set, bug reports will be sent to this [Sentry](https://sentry.io) DSN.                                         | not defined        |
| `RCH_SAASIFY_SECRET` | No        | If set, all requests must have a `x-saasify-proxy-secret` header set, equal to the value of `RCH_SAASIFY_SECRET`. | not defined        |

## REST API Documentation

Read docs on https://help.reacher.email/rest-api-documentation.

The API basically only exposes one endpoint: `POST /v0/check_email` expecting the following body:

```js
{
	"to_email": "someone@gmail.com",
	"from_email": "my@my-server.com", // optional, defaults to "user@example.org"
	"hello_name": "my-server.com",    // optional, defaults to "localhost"
	"proxy": {                        // optional, default is empty
		"host": "my-proxy.io",
		"port": "1080"
	}
}
```

Also check [`openapi.json`](./openapi.json) for the complete OpenAPI specification.

## License

`reacherhq/backend`'s source code is provided under a **dual license model**.

### Commercial license

If you want to use `reacherhq/backend` to develop commercial sites, tools, and applications, the Commercial License is the appropriate license. With this option, your source code is kept proprietary. Purchase an `reacherhq/backend` Commercial License at https://reacher.email/pricing.

### Open source license

If you are creating an open source application under a license compatible with the GNU Affero GPL license v3, you may use `reacherhq/backend` under the terms of the [AGPL-3.0](./LICENSE.AGPL).

[Read more](https://help.reacher.email/reacher-licenses) about Reacher's license.

## Sponsor my Open-Source Work

If you like my open-source work at Reacher, consider [sponsoring me](https://github.com/sponsors/amaurym/)! You'll also get 8000 free email verifications every month with your Reacher account, and a this contribution would mean A WHOLE LOT to me.
