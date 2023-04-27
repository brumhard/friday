# Tasks

## test

> Runs all tests in the workspace

```sh
cargo test --workspace
```

## fmt

> Formats the codebase.

This is using nightly rust to support unstable features.
See [rustfmt.toml](rustfmt.toml) for settings.

```sh
cargo +nightly fmt
```

## lint

> Runs the linter

**OPTIONS**

* ci
  * flags: --ci
  * type: bool
  * desc: enable ci mode to only fail on warnings

```sh
args="--workspace --all-targets --all-features"
if [ "$ci" == "true" ]; then
    args+=" -- -D warnings"
fi
cargo clippy $args
```

## audit

> Runs vulnerability scan

```sh
cargo audit
```

## vhs

> Generates docs/demo.gif from tape file

```shell
cargo build --bin friday --release
vhs docs/demo.tape
```

## docker

**OPTIONS**

* repo
  * flags: -r --repo
  * type: string
  * desc: repo to use for image
* tag
  * flags: -t --tag
  * type: string
  * desc: tag to use for image

> Builds the docker image

By default it takes the package version defined in the `Cargo.toml`.

```bash
set -eo pipefail
repo=${repo:-"ghcr.io/brumhard/friday"}
default_tag=$(yq -o json -r '.package.version' Cargo.toml)
tag=${tag:-$default_tag}
rust_version=$(yq -p toml -o json -r '.toolchain.channel' rust-toolchain ) 

docker build --build-arg "RUST_VERSION=$rust_version" -t "$repo:$tag" .
```

## ci

> Runs everyting needed in the CI

```sh
set -e
mask test
mask lint --ci
mask audit
```
