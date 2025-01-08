default:
    @just --list

gen-bindings:
    rm -rf bindings
    cargo test

gen-example-bindings: gen-bindings
    cargo test --example main
    rm -rf examples/main/bindings
    mv bindings examples/main/bindings

build-example:
    cd examples/main/web-app && bun run build

run-example: build-example
    RUST_LOG=debug cargo run --example main
