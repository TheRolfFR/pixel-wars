####################
#    BUILD RUST    #
####################
FROM rust:1.80 AS backend_build

# create a new empty shell project
RUN USER=root cargo new --bin backend
WORKDIR /backend

# copy over your manifests
COPY ./backend/Cargo.lock ./Cargo.lock
COPY ./backend/Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
COPY ./backend/src ./src
RUN rm ./target/release/deps/backend*
RUN cargo build --release


######################
#    BUILD SVELTE    #
######################
FROM node:20-alpine3.20 AS frontend_build

# install pnpm and dependencies
WORKDIR /frontend

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable

# copy your source tree
COPY ./frontend .

# build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

#####################
#    FINAL IMAGE    #
#####################
FROM rust:1.80

# copy frontend
WORKDIR /frontend
COPY --from=frontend_build /frontend/dist ./dist
COPY ./frontend/public/favicons ./public/favicons

# copy backend
RUN mkdir -p /backend
WORKDIR /backend
COPY --from=backend_build /backend/target/release/backend /usr/bin/backend

EXPOSE 80

# set the startup command to run your binary
CMD ["/usr/bin/backend"]
