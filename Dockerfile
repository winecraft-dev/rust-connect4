FROM rust:1.93

WORKDIR /app
COPY . .
RUN cargo build --release

COPY target/release/connect4 .

EXPOSE 8080

CMD [ "./connect4" ]
