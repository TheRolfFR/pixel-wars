<div align="center">
    <h1><img src="frontend/src/assets/logo-text.png" height="60" alt="rs/place"></h1>
    <em>A blazing fast r/place clone with Rust, Actix web, redis and Svelte</em>
</div>

## Requirements

These are not the required versions but what I used, your versions could be more recent.

- [Cargo](https://doc.rust-lang.org/cargo/) 1.80.0+
- [rustc](https://www.rust-lang.org/) 1.80.1+
- [NodeJS](https://nodejs.org/) 20.15.1+
- [pnpm](https://pnpm.io/) 9.9.0+
- [redis](https://redis.io/) 7.4.0+

## How to develop

To develop you will need to run frontend, backend, a redis server and configure [config.json](./config.json).

### Frontend and backend

Install pnpm packages:

```sh
cd frontend
pnpm i
```

To run backend and frontend in parallel, you can install [concurrently](https://www.npmjs.com/package/concurrently) globally:

```sh
pnpm i -g concurrently
```

Now you can launch dev command:

```sh
cd frontend
pnpm dev
```

### redis server

To run a redis server, just use docker or podman:

```sh
podman run -d -p 6379:6379 --name redis_server docker.io/library/redis:7-alpine
```

To watch the redis db, you can install and use [redis-commander](https://www.npmjs.com/package/redis-commander):

```sh
pnpm i -g redis-commander
```

### Configure config.json

To configure [config.json](./config.json), you have to configured as wished:

- ``redisUrl`` to your redis server URL you just launched. Value can be found or your machine or WSL ip address.
- ``host`` for the server IP exposed. Choose your local IP address for your house or keep localhost for your computer.

### Started URLs

Backend is started with reverse proxy for frontend at [http://localhost:8080/](http://localhost:8080/) while redis-commander is started at [http://127.0.0.1:8081/](http://127.0.0.1:8081/)

## How to build

Clone this repo duh. Build the docker image:

```sh
podman build -t rs-place:dev .
```

Modify the provided [docker-compose.yml](./docker-compose.yml) file to your liking. Don't forget to configure your [config.json](./config_prod.json) file with production values. ``debugMode`` must be false to start with static files from frontend build. Config properties ``redisUrl`` and ``host`` can be both overwritten by respectively the ``REDIS_URL`` and ``HOST`` environment variables.

You can then start the containers:

```sh
podman compose up -d
```

If you are using ports < 1024, you should stop your podman machine and grant it root permissions:

```sh
podman machine stop
podman machine set --rootful
podman machine start
```
