#!/bin/sh

# Just some simple E2E tests using the HTTP server. The majority of the tests
# are performed in check-if-email-exists upstream. These ones here are to test
# that the HTTP requests are correctly set up.
# See https://github.com/amaurymartiny/check-if-email-exists/tree/master/test_suite
# FIXME Switch to a more robust test suite.

set -e

# Run reacher, assumes the binary is in ./target/release
./target/release/reacher &

# Do some simple curl requests

# Test foo@bar
expected='{"input":"foo@bar","misc":{"is_disposable":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":null,"domain":"","is_valid_syntax":false,"username":""}}'
actual=$(curl -X POST -H 'Content-Type: application/json' -d '{"to_email": "foo@bar"}' localhost:8080/check_email)
if [ "$expected" != "$actual" ]; then
  echo "E2E test failed, got:"
  echo $actual
  exit 1
fi


# Test foo@bar.baz
expected='{"input":"foo@bar.baz","misc":{"is_disposable":false},"mx":{"accepts_mail":false,"records":[]},"smtp":{"can_connect_smtp":false,"has_full_inbox":false,"is_catch_all":false,"is_deliverable":false,"is_disabled":false},"syntax":{"address":"foo@bar.baz","domain":"bar.baz","is_valid_syntax":true,"username":"foo"}}'
actual=$(curl -X POST -H 'Content-Type: application/json' -d '{"to_email": "foo@bar.baz"}' localhost:8080/check_email)
if [ "$expected" != "$actual" ]; then
  echo "E2E test failed, got:"
  echo $actual
  exit 1
fi
