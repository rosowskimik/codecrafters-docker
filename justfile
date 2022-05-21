#!/usr/bin/env -S just --justfile

target := '/tmp/docker-c.out'

@build:
    docker build -t mydocker .

@docker *ARGS: 
    docker run --cap-add="SYS_ADMIN" mydocker {{ARGS}}

cargo *ARGS:
    @cargo build --quiet --release
    sudo target/release/docker-starter-rust {{ARGS}}