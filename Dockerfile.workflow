# Intended for use within CI, use the normal Dockerfile for local builds

FROM bash:latest as fetcher
ARG TARGETPLATFORM
ARG BUILDPLATFORM

WORKDIR /
COPY ./artifacts ./artifacts

RUN bash -l -c 'case $TARGETPLATFORM in \

    "linux/amd64") \
        mv ./artifacts/x86_64-unknown-linux-musl/wgg_http / \
        ;; \

    "linux/arm64") \
        mv ./artifacts/aarch64-unknown-linux-gnu/wgg_http / \
        ;; \

    "linux/arm/v7") \
        mv ./artifacts/armv7-unknown-linux-gnueabihf/wgg_http / \
        ;; \
    esac'

FROM alpine:3.11

COPY --from=fetcher /wgg_http /
COPY ./dist /static

ENV WGG__APP_SETTINGS__dist_dir="/static"
ENV WGG__APP_SETTINGS__APPDATA_DIR="/appdata"
ENV WGG__DB_SETTINGS__SQLITE__DB_PATH="/appdata/wgg.db"

EXPOSE 8080

CMD ["./wgg_http"]