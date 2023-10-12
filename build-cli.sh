#!/bin/bash

TAG=$(cat Cargo.toml | grep '^version =' | cut -d " " -f3 | xargs)

# rustup target add x86_64-unknown-linux-musl
rm -rf target
cargo build --release --target x86_64-unknown-linux-musl --features 'cli'
cp target/x86_64-unknown-linux-musl/release/nioca-client out/amd64/nioca-client-$TAG

# rustup target add aarch64-unknown-linux-musl
rm -rf target
cross build --release --target aarch64-unknown-linux-musl --features 'cli'
cp target/aarch64-unknown-linux-musl/release/nioca-client out/aarch64/nioca-client-$TAG

# rustup target add x86_64-apple-darwin
#rm -rf target
#cargo build --release --target x86_64-apple-darwin --features 'cli'
#cp target/x86_64-unknown-linux-musl/release/nioca-client out/osx/nioca-client-$TAG

# rustup target add x86_64-pc-windows-msvc
#rm -rf target
#cargo build --release --target x86_64-pc-windows-msvc --features 'cli'
#cp target/x86_64-unknown-linux-musl/release/nioca-client out/windows/nioca-client-$TAG
