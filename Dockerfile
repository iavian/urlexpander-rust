FROM rust:1.57 as builder

RUN USER=root cargo new --bin rust-docker-web

WORKDIR /rust-docker-web
COPY ./Cargo.toml ./Cargo.toml
RUN rm src/*.rs
ADD . ./

RUN cargo build --release
RUN ls

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

COPY --from=builder /rust-docker-web/target/release/url-resolver ${APP}/url-resolver

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./url-resolver"]
