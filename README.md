<div align="center">
    <h1><br />
        <img src="frontend/src/assets/logo-text.png" height="60" alt="rs/place">
    <br /><br /></h1>

<em>A blazing fast r/place clone with Rust, Actix web, redis and Svelte</em>
</div>

## Requirements

These are not the required versions but what I used, your versions could be more recent.

- Cargo 1.80.0+
- rustc 1.80.1+
- NodeJS 20.15.1+
- pnpm 9.9.0+
- concurrently 8.2.2+
- redis 7.4.0

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

To build, you can just build the docker image:

```sh
podman build -t rs-place:dev .
```

You can then create a docker compose file to launch your image just built and a redis server:

```yaml
services:
  redis:
    image: 'docker.io/redis:7-alpine'
    restart: always
    ports:
      - '6379:6379'
    command: redis-server /redis.conf
    volumes:
      - './redis/redis.conf:/redis.conf'
      - './redis/data:/data'
    # So that you can use in config_prod.json redis://rsplace_redis/
    hostname: rsplace_redis
    container_name: rsplace_redis
  place:
    image: 'rs-place:dev'
    restart: always
    ports:
      - '80:80'
    volumes:
      - './config_prod.json:/config.json'
    container_name: rsplace_server
```

```sh
podman compose up -d
```

If you are using ports < 1024, you should stop your podman machine and grant it root permissions:

```sh
podman machine stop
podman machine set --rootful
podman machine start
```

Don't forget to configure your [config.json](./config_prod.json) file with production values. ``debugMode`` must be false to start with static files from frontend build.

Config properties ``redisUrl`` and ``host`` can be both overwritten by respectively the ``REDIS_URL`` and ``HOST`` environment variables.
