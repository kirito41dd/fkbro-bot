FROM rust:latest AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:buster-slim
COPY --from=build /app/target/release/fkbro-bot /bin/fkbro-bot
COPY --from=build /app/templates /app/templates

WORKDIR /app
ENTRYPOINT ["fkbro-bot"]