#syntax=docker/dockerfile:1.5.2

FROM --platform=$TARGETPLATFORM rust:1.68.2 as base
WORKDIR /somewhere
ENV CARGO_HOME="/cache/cargo/"
ENV BUILD_TARGET="/somewhere/target"
COPY Cargo.toml .
COPY Cargo.lock .
# hack to only build deps
RUN --mount=type=cache,target=$CARGO_HOME \
    --mount=type=cache,target=$BUILD_TARGET <<EOF
    mkdir src
    echo 'fn main() {}' > src/main.rs
    cargo build --release
    rm -rf src
EOF

FROM base as build
COPY src/ src/
RUN --mount=type=cache,target=$CARGO_HOME \
    --mount=type=cache,target=$BUILD_TARGET <<EOF
    cargo build --release --bin server
    cp $BUILD_TARGET/release/server /
EOF

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc:nonroot
COPY --from=build /server /

ENTRYPOINT [ "/server" ]
