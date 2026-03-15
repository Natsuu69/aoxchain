# aoxcrpc

## Purpose

`aoxcrpc` is responsible for the **API ingress layer (HTTP/gRPC/WebSocket)** domain within the AOXChain workspace.

## Code Scope

- `proto/`
- `src/middleware/`
- `src/grpc/`
- `src/http/`
- `src/websocket/`
- `src/config.rs`

## Operational Notes

- API and behavior changes should be evaluated for backward impact.
- Prefer explicit parameters over implicit defaults in critical paths.
- Security-impacting changes in this crate should be accompanied by test/example updates.

## Local Validation

```bash
cargo check -p aoxcrpc
```

## Related Components

- Top-level architecture: [`../../README.md`](../../README.md)
- Crate catalog: [`../README.md`](../README.md)
