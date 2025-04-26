default:
    @just --list

# Generate bindings
bindings:
    rm -rf bindings
    rm -rf packages/nitram/bindings
    # Generate main bindings
    cargo test
    cp -r bindings packages/nitram/bindings
    # Generate example bindings
    cargo test --example main
    # Move all bindings to example
    rm -rf examples/main/bindings
    mv bindings examples/main/bindings

install-example:
    cd examples/main/web-app && bun install

build-example:
    cd examples/main/web-app && bun run build

run-example: build-example
    RUST_LOG=debug cargo run --example main

pack:
    cd packages/nitram && bun pm pack
