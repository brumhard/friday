#syntax=docker/dockerfile:1.5.2

# this should be set to the value from rust-toolchain
ARG RUST_VERSION=invalid

FROM --platform=$TARGETPLATFORM rust:${RUST_VERSION}-slim as base
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
COPY crates/ crates/
RUN --mount=type=cache,target=$CARGO_HOME \
    --mount=type=cache,target=$BUILD_TARGET <<EOF
    cargo build --release --bin fridaypi
    cp $BUILD_TARGET/release/fridaypi /
EOF

FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc:nonroot
COPY --from=build /fridaypi /

ENTRYPOINT [ "/fridaypi" ]
