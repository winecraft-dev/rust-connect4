FROM rust:1.93 AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app

COPY --from=builder /app/target/release/connect4 .

EXPOSE 8080

CMD [ "./connect4" ]
