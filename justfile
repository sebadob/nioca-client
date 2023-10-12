set shell := ["bash", "-uc"]

export TAG := `cat Cargo.toml | grep '^version =' | cut -d " " -f3 | xargs`


# run `cargo clippy` with correct env vars
clippy:
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    cargo clippy --features "cli"
    cargo clippy --features "actix"
    cargo clippy --features "axum"


# runs the application
run command="-h":
    #!/usr/bin/env bash
    set -euxo pipefail
    clear

    cargo run --features "cli" -- {{ command }}


## runs the UI in development mode
#run-ui:
#    #!/usr/bin/env bash
#    cd frontend
#    npm run dev -- --host


# prints out the currently set version
version:
    echo $TAG


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


build-cli: clippy
    #!/usr/bin/env bash
    set -euxo pipefail

    # manually update the cross image: docker pull ghcr.io/cross-rs/x86_64-unknown-linux-musl:main
    which cross || echo "'cross' needs to be installed: cargo install cross --git https://github.com/cross-rs/cross"

    cross build --features "cli" --release --target x86_64-unknown-linux-musl
    cp target/x86_64-unknown-linux-musl/release/nioca-client out/


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
