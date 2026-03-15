# AOXChain Crate Catalog

This document summarizes **responsibility boundaries** and entry points for crates in the workspace.

## Core Protocol

- [`aoxcore`](aoxcore/README.md): identity, genesis, transactions, mempool, and state primitives
- [`aoxcunity`](aoxcunity/README.md): consensus flow (quorum/vote/fork-choice/seal)
- [`aoxcvm`](aoxcvm/README.md): multi-lane execution and VM compatibility

## Networking and API

- [`aoxcnet`](aoxcnet/README.md): discovery/gossip/sync/transport
- [`aoxcrpc`](aoxcrpc/README.md): HTTP, gRPC, and WebSocket API layer
- [`aoxcsdk`](aoxcsdk/README.md): integration-focused SDK surface

## Operations and Tooling

- [`aoxcmd`](aoxcmd/README.md): node lifecycle, economics, bootstrap, and smoke commands
- [`aoxckit`](aoxckit/README.md): keyforge, certificate, and identity tooling
- [`aoxconfig`](aoxconfig/README.md): configuration models

## Supporting Crates

- [`aoxcai`](aoxcai/README.md)
- [`aoxcdata`](aoxcdata/README.md)
- [`aoxcexec`](aoxcexec/README.md)
- [`aoxcenergy`](aoxcenergy/README.md)
- [`aoxclibs`](aoxclibs/README.md)
- [`aoxcmob`](aoxcmob/README.md)
- [`aoxcontract`](aoxcontract/README.md)
- [`aoxchal`](aoxchal/README.md)

## Governance Rule

Any PR that changes a crate responsibility boundary should include:

1. README revision for the affected crate
2. Required test/example updates
3. Backward-compatibility notes (if applicable)
