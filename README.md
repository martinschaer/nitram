# Nitram

Opinionated RPC server for Rust and Typescript.

## To do:

- fix types (TODO in packages/nitram/index.ts)
- handle error in client when a requests is not queued because of a duplicate
- test queued requests
- Quick start guide
- TODO(6cd5): better DBSessionId

## Publishing:

```sh
# -- NPM
cd packages/nitram
# bun pm pack --dry-run
# bun pm pack
# bun publish --dry-run
bun publish
cd ../..

# -- Crate
# cargo publish --dry-run
cargo publish
```
