FROM restreamio/gstreamer:latest-dev as builder
RUN apt update && apt install build-essential curl -y

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN USER=root cargo new --bin ahps

WORKDIR /ahps

RUN touch src/lib.rs
RUN mv src/main.rs src/bin.rs

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY . ./

RUN rm ./target/release/deps/ahps*
RUN rm ./target/release/deps/lib*
RUN cargo build --release


FROM ubuntu:20.10

ARG APP=/usr/src/app

COPY --from=builder /ahps/target/release/ahps ${APP}/ahps

WORKDIR ${APP}

CMD ["./ahps"]