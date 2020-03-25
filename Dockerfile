FROM rust:1.42-buster AS build

RUN apt-get update \
 && apt-get install -y clang libclang-dev libsqlite3-dev llvm-dev \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /scum-bot/

RUN mkdir -p /scum-bot/src/ \
 && echo 'fn main() {}' > /scum-bot/src/main.rs

ADD ./Cargo.toml ./Cargo.lock /scum-bot/

RUN cargo build --release

ADD ./src/ /scum-bot/src/

RUN touch /scum-bot/src/main.rs \
 && cargo build --release

FROM debian:buster

RUN apt-get update \
 && apt-get install -y sqlite3 libsqlite3-dev

COPY --from=build /scum-bot/target/release/scum_bot /opt/scum-bot/bin/scum_bot

ADD ./config/sql/* /opt/scum-bot/share/sql/

ADD ./config/bin/* /opt/scum-bot/bin/

CMD ["/opt/scum-bot/bin/entrypoint.sh"]
