FROM rust:bullseye AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye

WORKDIR /app

COPY --from=builder /app/target/release/connect4 .

EXPOSE 8080

CMD [ "./connect4" ]
