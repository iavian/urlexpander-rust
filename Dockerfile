FROM rust:1.51 as builder

RUN USER=root cargo new --bin rust-docker-web

WORKDIR /rust-docker-web
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs
 
RUN ls -ltr
RUN pwd

ADD . ./

RUN cargo build --release

RUN ls -ltr ./target/release
RUN pwd

FROM debian:buster-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

EXPOSE 8080

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

RUN pwd
RUN ls -ltr /home
COPY --from=builder /rust-docker-web/target/release/url-resolver ${APP}/url-resolver

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./url-resolver"]
