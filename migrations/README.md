# Database and Migrations

## Usage

See https://github.com/launchbadge/sqlx/blob/main/sqlx-cli/README.md

## `sqlxmq` migrations

The following migration files have been copied from the [sqlxmq repo](https://github.com/Diggsey/sqlxmq) as per the [given instructions](https://github.com/Diggsey/sqlxmq/blob/6d3ed6fb99e7592e370a7f3ec074ce0bebae62fd/README.md?plain=1#L111):

- `20210316025847_setup.{up,down}.sql`
- `20210921115907_clear.{up,down}.sql`
- `20211013151757_fix_mq_latest_message.{up,down}.sql`
- `20220208120856_fix_concurrent_poll.{up,down}.sql`
- `20220713122907_fix-clear_all-keep-nil-message.{up,down}.sql`

## Reacher migrations

The following migrations are specific to Reacher:

- `20220117025847_email_data.down.sql`: set up the `bulk_jobs` and `email_result` tables
- `20220810141100_result_created_at.down.sql`: add a `created_at` column  on `email_result`
