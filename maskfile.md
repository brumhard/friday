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

By default it tries to take the latest tag and use that as a tag.
If that is not found it uses `latest` instead.

```bash
repo=${repo:-"ghcr.io/brumhard/friday"}
default_tag=$(git describe --tags --abbrev=0 2>/dev/null || echo latest)
tag=${tag:-$default_tag}

docker build -t "$repo:$tag" .
```

## ci

> Runs everyting needed in the CI

```sh
set -e
mask test
mask lint --ci
mask audit
```
