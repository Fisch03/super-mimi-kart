FROM rust:slim AS rust

FROM rust AS base
RUN cargo install cargo-chef 

ENV SKIP_CLIENT_BUILD=true
WORKDIR /usr/src/super-mimi-kart

FROM base AS web-builder 
# install tools
RUN rustup target add wasm32-unknown-unknown
RUN cargo install wasm-pack

# prepare deps
FROM base AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# compile game client
FROM web-builder AS build-game
COPY . .
RUN wasm-pack build --target web --release ./game

#compile editor
FROM web-builder AS build-editor
COPY . .
RUN wasm-pack build --target web --release ./editor

FROM base AS build-server
COPY --from=plan /usr/src/super-mimi-kart/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# compile server
RUN cargo build --bin super-mimi-kart --release

# get compiled web stuff
COPY --from=build-game /usr/src/super-mimi-kart/game/pkg static/game
COPY --from=build-editor /usr/src/super-mimi-kart/editor/pkg static/editor

FROM debian:bookworm-slim AS runtime
WORKDIR /super-mimi-kart
COPY --from=build-server /usr/src/super-mimi-kart/target/release/super-mimi-kart super-mimi-kart
COPY --from=build-server /usr/src/super-mimi-kart/static static
COPY ./maps maps

EXPOSE 8080
CMD ["./super-mimi-kart"]
