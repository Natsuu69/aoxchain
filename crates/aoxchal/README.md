# aoxchal

Hardware abstraction layer (HAL) facade for AOXChain.

## Scope
- CPU capability detection for crypto fast paths.
- Memory region facade for future mmap-backed allocators.

## Status
Current implementation provides deterministic, portable placeholders that compile in all workspace targets and can be replaced with platform-specific backends incrementally.
