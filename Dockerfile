FROM rust:1.51.0 as builder

RUN apt-get update && apt-get install build-essential libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev -y
    
RUN USER=root cargo new --bin ahps

WORKDIR /ahps

RUN touch src/lib.rs && mv src/main.rs src/bin.rs

COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY . ./

RUN rm ./target/release/deps/ahps* && rm ./target/release/deps/lib*
RUN cargo build --release


FROM debian:buster-slim
RUN apt-get update && apt-get install build-essential libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev \
    gstreamer1.0-plugins-base gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav libgstrtspserver-1.0-dev libges-1.0-dev -y

ARG APP=/usr/src/app

ENV APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /ahps/target/release/ahps ${APP}/ahps

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./ahps"]