#!/bin/sh
#
# DON'T EDIT THIS!
#
# CodeCrafters uses this file to test your code. Don't make any changes here!
#
# DON'T EDIT THIS!
exec cargo run \
    --release \
    --target-dir=/tmp/codecrafters-docker-target \
    --manifest-path "$(dirname "$0")/Cargo.toml" "$@"
# exec /tmp/codecrafters-docker-target/release/docker-starter-rust "$@"