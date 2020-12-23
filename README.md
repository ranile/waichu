# Waichu

A messaging app built in pure Rust. It uses [`warp`](https://github.com/seanmonstar/warp) on the backend and [`Yew`](https://github.com/yewstack/yew/) on the frontend.
It uses `PostgreSQL` as the database.

You can find a hosted version [here](https://waichu.31416.dev/)

## Building and deploying

You can build this app in 2 ways:
1. Building the docker image
2. Manually building the app

### Building the docker image
The docker build process will install all the tools required and build the app.
```shell
docker build -t waichu .
```

Once the build completes, you can run the image as:
```shell
docker run -d --network host --name waichu --env "DATABASE_URL=postgresql://waichu:password@localhost:5432/waichu" waichu
```

### Manually building the app 

In order to build the app manually, following tools must be installed:
- [Rust](https://www.rust-lang.org)
- [`trunk`](https://github.com/thedodd/trunk)

```shell
trunk build frontend/Trunk.toml --release --dist ./dist
DIST_DIR=./dist DATABASE_URL=postgresql://waichu:password@localhost:5432/waichu cargo build -p backend --release
```

### Options

| Name           | Required               | Description                                                              | Default |
|----------------|------------------------|--------------------------------------------------------------------------|---------|
| `DATABASE_URL` | ✅                      | The path at which your instance of  `PostgreSQL` is running              |         |
| `PORT`         | ❌                      | The port to run the server on                                            | 9090    |
| `DIST_DIR`     | only outside of docker | The path where frontend static files are (must **not** be set in docker) |         |


## Contributions

Your contributions are welcome.

### Running tests

#### Backend tests

```shell
TEST_DATABASE_URL=postgresql://waichu:password@localhost:5432/waichu cargo test --release -p backend -- --test-threads 1
```
where `TEST_DATABASE_URL` is the path where your instance of `PostgreSQL` is running. You should use a separate database for testing as it'll be wiped before every run.

It is important to note that these tests must be run sequentially as they make database queries. 
