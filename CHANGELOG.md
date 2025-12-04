# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.3] - 2025-12-04

### Added

- Add configurable WebSocket options:
  - `ping_interval_in_seconds` — configurable ping interval for WebSocket keepalive (default = 30s)
  - `timeout_in_seconds` — configurable connection timeout when no pong/keepalive is received (default = 90s)
  - `max_frame_size` — configurable maximum WebSocket frame size (default = 128 KiB)

- Add NoResponse nice message to use when a server message handler does not need to send a message to the client.
