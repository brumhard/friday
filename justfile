set shell := ["bash", "-uc"]
set ignore-comments

default:
    just --list

alias t := test
# test everything
test:
    cargo test --workspace

fmt:
    # extra unstable options are enabled here
    # see https://github.com/rust-lang/rustfmt/issues/5511
    cargo fmt -- \
        --config wrap_comments=true \
        --config reorder_imports=true \
        --config imports_layout=HorizontalVertical \
        --config imports_granularity=Crate \
        --config group_imports=StdExternalCrate \
        --config format_code_in_doc_comments=true \
        --config format_macro_matchers=true \
        --config format_macro_bodies=true \
        --config blank_lines_upper_bound=1 \
        --config condense_wildcard_suffixes=true \
        --config use_field_init_shorthand=true \
        --config use_try_shorthand=true \
        --config use_small_heuristics=max \


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