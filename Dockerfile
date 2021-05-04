FROM rust:1.51.0 as builder
RUN USER=root cargo new --bin ahps
WORKDIR /ahps
RUN touch ./src/lib.rs 
RUN mv ./src/main.rs ./src/bin.rs 
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm -r src/*

ADD . ./

RUN rm ./target/release/deps/ahps*
RUN cargo build --release

FROM restreamio/gstreamer:latest-prod
ARG APP=/usr/src/ahps
COPY --from=builder /ahps/target/release/ahps ${APP}/ahps
WORKDIR ${APP}
CMD ["./ahps"]