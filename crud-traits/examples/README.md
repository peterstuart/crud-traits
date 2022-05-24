# Examples

## Setup

Set up an empty PostgreSQL database. You can optionally use the `docker-compose.yml` file in this directory to do that, by running:

```shell
docker-compose up -d
```

Set the `DATABASE_URL` environment variable to point at that database. If you used the `docker-compose.yml` file, that will be:

```shell
export DATABASE_URL=postgres://crud-traits:password@localhost/crud-traits
```

## Running

```shell
cargo run --example demo
```
