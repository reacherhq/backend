[![Actions Status](https://github.com/reacherhq/microservices/workflows/CI/badge.svg)](https://github.com/reacherhq/microservices/actions)
![GitHub](https://img.shields.io/github/license/reacherhq/microservices.svg)

# Reacher Microservices

This package deploys the [`check-if-email-exists`](https://github.com/reacherhq/check-if-email-exists) function as a lambda function on AWS via [serverless](https://serverless.com/).

It serves as the backend for https://reacherhq.github.io.

To try it out, follow the steps below:

#### Set up `serverless`

Follow serverless's guide: https://serverless.com/framework/docs/providers/aws/guide/quick-start/.

#### Invoke the function locally

Change `put_your_email_here@gmail.com` to the email your wish to test inside `./test/payload.json`, and run from the root folder:

Note: you need to have Docker installed.

```bash
serverless invoke local -f check-if-email-exists-serverless -d "$(cat test/payload.json)"
```

#### Deploy the function to AWS

```bash
serverless deploy
```

#### Invoke the deployed function

```bash
serverless invoke -f check-if-email-exists-serverless -d "$(cat test/payload.json)"
```
