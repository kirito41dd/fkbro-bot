FROM rust:1.59.0 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
COPY --from=build /app/target/release/fkbro-bot /bin/fkbro-bot
COPY --from=build /app/templetes /app/templetes

WORKDIR /app
ENTRYPOINT ["fkbro-bot"]