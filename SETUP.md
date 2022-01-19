## Dev environment installation

1. Install [docker](https://docs.docker.com/get-docker/)
2. Run the following command to get a postgresql database running - `docker run --name <container-name> -p 5432:5432 -e POSTGRES_PASSWORD=<password> -d postgres:14`. Note that default user and database is postgres.
3. Download migrations from [sqlxmq](https://github.com/Diggsey/sqlxmq#database-schema) to setup database for message queue. 
4. Install [psql](https://blog.timescale.com/blog/how-to-install-psql-on-mac-ubuntu-debian-windows/) to apply migrations to db.
5. Use `cargo install rargs` and run the following to apply migrations - `ls migrations/sqlxmq/*up.sql | rargs psql postgres://postgres:<password>@localhost/postgres -f migrations/{0}`
6. Add a `.env` file with a single key `PGPASSWORD=<password>`. This will be read by the application at runtime from the environment and be used to connect to the environment.
