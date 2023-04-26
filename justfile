set shell := ["bash", "-uc"]
set ignore-comments

default:
    just --list

alias t := test
# test everything
test:
    cargo test --workspace

# lint everything
lint $mode="":
    #!/usr/bin/env bash
    args="--workspace"
    if [ "$mode" == "ci" ]; then
        args="--workspace --all-targets --all-features -- -D warnings"
    fi
    cargo clippy $args

fix:
    cargo fix --workspace --allow-dirty

# run vulnerability scan
audit:
    cargo audit

# generate demo gif
vhs:
    cargo build --bin friday --release
    vhs docs/demo.tape

docker_repo := "github.com/brumhard/friday"
docker_tag := `git describe --tags --abbrev=0 2>/dev/null || echo latest`
# build docker image
docker tag=docker_tag repo=docker_repo:
    docker build -t {{repo}}:{{tag}} .

# run everything ci related
ci: test (lint "ci") audit