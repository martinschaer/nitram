# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Overview

Nitram is an opinionated RPC server library for Rust with a TypeScript client. It is published as both a Rust crate (`nitram`) and an npm package (`nitram`).

## Commands

### Rust (Crate)

```sh
cargo build
cargo test                    # Run all tests
cargo test --test nitram      # Run the integration tests in tests/nitram.rs
cargo test --example main     # Run example tests (generates TS bindings)
```

### TypeScript (packages/nitram)

```sh
bunx @biomejs/biome check --write packages/nitram   # Lint and format
bunx @biomejs/biome format --write packages/nitram  # Format only
```

### Just recipes

```sh
just bindings        # Regenerate all TypeScript bindings (runs cargo test, copies to packages/ and examples/)
just run-example     # Build the web-app and run the main example on :8000
just build-example   # Build examples/main/web-app
just install-example # Install web-app dependencies
just pack            # Pack the npm package
```

### Publishing

```sh
# npm
cd packages/nitram && bun publish

# crate
cargo publish
```

## Architecture

### Rust Library (`src/`)

The library wraps `rpc-router` and `actix-ws` to provide a structured WebSocket RPC server.

**Core flow:**
1. `NitramBuilder` registers handlers as public, private, or server-message handlers, then `build()` produces a `Nitram` instance.
2. `Nitram` (Arc-cloned into Actix app data) holds `NitramInner` (session map) and three `rpc-router` routers.
3. `ws::handler` is the Actix WebSocket endpoint. It spawns three tasks: ping/timeout loop, server-messages push loop, and message receive loop.
4. On each incoming text message, `Nitram::send()` deserializes to `NitramRequest`, routes to the appropriate handler, and returns a serialized `NitramResponse`.

**Handler types:**
- **Public**: accessible without authentication; receives `WSSessionAnonymResource`.
- **Private**: requires authentication; receives `WSSessionAuthedResource` and a per-session `Store`.
- **Server-message**: polled on an interval per authenticated session for subscribed topics; return `MethodError::NoResponse` to suppress sending.

**Auth model:** Sessions start as `NitramSession::Anonymous`. Calling `NitramInner::auth_ws_session()` inside an `authenticate_handler` transitions them to `NitramSession::Authenticated`, which carries a `DBSession`, a topic subscription map, and a `Store` (per-session key-value store).

**TypeScript binding generation:** `ts-rs` exports types to `bindings/`. The `nitram_handler!` macro defines both the params struct (exported to `API/Params.ts`) and the API shape struct (exported to `API/index.ts`). Run `just bindings` to regenerate after changes.

**Key files:**
- `src/builder.rs` — `NitramBuilder`
- `src/nitram.rs` — `Nitram` and `NitramInner`
- `src/ws.rs` — Actix WebSocket handler
- `src/auth.rs` — Session types, token generation/parsing, `AuthenticateParams`
- `src/models.rs` — `DBSession`, `Store`, `AuthStrategy`
- `src/messages.rs` — `NitramRequest`, `NitramResponse`, `NitramServerMessage`
- `src/lib.rs` — Public API + `nitram_handler!` / `nitram_api!` macros

### TypeScript Client (`packages/nitram/`)

`packages/nitram/index.ts` exports the `Server` class. It:
- Manages a WebSocket connection with auto-reconnect.
- Queues requests when disconnected (deduplicates by content hash).
- Dispatches responses by request ID.
- Handles server-push messages (topic-based) via `addServerMessageHandler`.
- Authenticates via `server.auth(token)` on connect, storing the token in `localStorage`.

TypeScript type bindings live in `packages/nitram/bindings/` (copied from Rust's `bindings/` by `just bindings`).

### Example (`examples/main/`)

`examples/main/main.rs` is a self-contained Actix server demonstrating the full integration: public handlers (`GetToken`, `Authenticate`), private handlers (`SendMessage`, `GetUser`), and a server-message handler (`Messages`). The frontend lives in `examples/main/web-app/` (Vite app).
