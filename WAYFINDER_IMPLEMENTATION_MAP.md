# WAYFINDER IMPLEMENTATION MAP: Loro Integration

## Destination

The full integration of `loro` CRDTs into `@sheetah-model/`, utilizing an ECS-inspired architecture for high-performance data processing, providing seamless synchronization between local state and a durable DB via a single-threaded, snapshot+delta event loop.

## Notes

- **Themes**: ECS (Entity Component System), DB (Database), CRDT (Loro).
- **The Model**: The entire domain model resides within `Loro` documents.
- **Architecture**: A reactive pipeline where `Loro` change events drive the `DB`, and the `DB` state is projected into an `ECS` for high-performance runtime operations and rendering.
- **Concurrency**: Single-threaded event loop to ensure strict ordering and prevent desynchronization.
- **Performance**: Use `ECS` (AoS $\rightarrow$ SoA) for repeated computations and high-FPS rendering.
- **Persistence**: Snapshot + Delta model.

## Decisions so far

<!-- The index of resolved tickets will live here -->

## Not yet specified

- **Optimistic Reconciliation**: Algorithms for reconciling a high-speed local `ECS` view with the authoritative, potentially delayed, `DB`-backed `Loro` state.
- **The Projection Layer**: The mechanism for efficiently projecting nested `Loro` structures into flat `ECS` component arrays.
- **DB-to-ECS Delta Flow**: How to translate `DB` change events into incremental `ECS` component updates to avoid full-state re-projections.
- **Lifecycle Management**: Synchronizing entity existence (creation/deletion) between the `Loro` document structure and the `ECS` entity registry.

## Out of scope

- Multi-threaded/Sharded worker pools.
- Centralized "Authority" models.
- Complex distributed consensus protocols (relying on CRDT convergence instead).
