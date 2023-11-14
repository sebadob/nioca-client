set shell := ["bash", "-uc"]

export TAG := `cat Cargo.toml | grep '^version =' | cut -d " " -f3 | xargs`


# run `cargo clippy` with correct env vars
clippy:
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    cargo clippy --lib --no-default-features --features hydrate
    cargo clippy --bin=nioca-client --no-default-features --features ssr
#    cargo clippy --features "actix"
#    cargo clippy --features "axum"
#    cargo clippy --features "cli"
#    cargo clippy --features "generic"

# build the application in development mode
build-cli:
    clear && DEV_MODE=true cargo leptos build

# build the application in release mode
build-cli-release:
    clear && DEV_MODE=false cargo leptos build -r -P

# prints out the currently set version
version:
    echo $TAG

# runs the application
run command="-h": build-cli
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    DEV_MODE=true ./target/debug/nioca-client {{ command }}

## runs the full set of tests
#test:
#    #!/usr/bin/env bash
#    set -euxo pipefail
#    clear
#
#    #DATABASE_URL={{db_url}} cargo build
#    #DATABASE_URL={{db_url}} cargo run &
#    #sleep 1
#    #PID=$(echo "$!")
#    #echo "PID: $PID"
#
#    DATABASE_URL={{db_url}} cargo test
#    #kill "$PID"
#    echo All tests successful


build: clippy
    #!/usr/bin/env bash
    set -euxo pipefail

    # manually update the cross image: docker pull ghcr.io/cross-rs/x86_64-unknown-linux-musl:main
    which cross || echo "'cross' needs to be installed: cargo install cross --git https://github.com/cross-rs/cross"

    mkdir -p out/x86_64
    mkdir -p out/aarch64
    mkdir -p out/windows
    #mkdir -p out/mac_os

    # linux x86 musl
    cross build --features "cli" --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/nioca-client out/x86_64/nioca-client-$TAG

    # linux aarch64 musl
    cross build --features "cli" --release --target aarch64-unknown-linux-musl
    cp target/aarch64-unknown-linux-musl/release/nioca-client out/aarch64/nioca-client-$TAG

    # windows
    cross build --features "cli" --release --target x86_64-pc-windows-gnu
    cp target/x86_64-pc-windows-gnu/release/nioca-client.exe out/windows/nioca-client-$TAG.exe

    # TODO mac os currently does not work because of ring: https://github.com/briansmith/ring/issues/1442
    # mac os
    #cargo build --features "cli" --release --target x86_64-apple-darwin
    #cp target/x86_64-apple-darwin/release/nioca-client.exe out/mac_os/nioca-client-$TAG.exe


# makes sure everything is fine
is-clean:
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    # exit early if clippy emits warnings
    cargo clippy --features "cli" -- -D warnings
    cargo clippy --features "actix" -- -D warnings
    cargo clippy --features "axum" -- -D warnings

    # make sure everything has been committed
    git diff --exit-code

    echo all good


# sets a new git tag and pushes it
release: is-clean
    #!/usr/bin/env bash
    set -euxo pipefail

    git tag "v$TAG"
    git push origin "v$TAG"


## publishes the application images
#publish: build-image
#    docker build --no-cache -f Dockerfile -t sdobedev/nioca:$TAG .
#    docker push sdobedev/nioca:$TAG
#    docker tag sdobedev/nioca:$TAG ghcr.io/sebadob/nioca:$TAG
#    docker push ghcr.io/sebadob/nioca:$TAG
#
#    docker tag sdobedev/nioca:$TAG sdobedev/nioca:latest
#    docker push sdobedev/nioca:latest
#    docker tag sdobedev/nioca:latest ghcr.io/sebadob/nioca:latest
#    docker push ghcr.io/sebadob/nioca:latest
