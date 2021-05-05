FROM restreamio/gstreamer:latest-dev as builder
RUN apt-get update && apt-get install -y curl
RUN apt-get install build-essential -y
 
RUN mkdir -p /user/rust-builder/src
WORKDIR /user/rust-builder/src
 
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
WORKDIR /ahps
COPY . .
RUN cargo build --release

FROM alpine:latest
RUN apk update && apk add bash
COPY --from=builder /ahps/target/release/ahps /ahps/ahps

WORKDIR /ahps