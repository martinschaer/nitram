default:
    @just --list

build-example:
    cd examples/web-app && bun run build

run-example: build-example
    cargo run --example main
