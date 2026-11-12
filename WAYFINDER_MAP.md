# WAYFINDER MAP

## Destination

A technical specification and architectural blueprint for making `@sheetah-model/` CRDT-aware using `loro`, enabling full synchronization of Workspaces and their constituent Tables, with a primary focus on user experience (smoothness) and eventual consistency with error surfacing.

## Notes

- Loro is the single source of truth.
- Data is extracted from Loro for computation and database updates.
- Syncing happens via Loro documents.
- Synchronization is event-based, utilizing a dedicated thread with async concurrency.
- Every Table is a document; the Workspace is also a document.
- Focus on using the latest valid version of a document while surfacing errors.

## Decisions so far

<!-- the index: one line per closed ticket, enough to judge relevance, then zoom the link for the detail the ticket holds -->

## Progress

### Done
- [x] Defined the project destination and created the wayfinder map via `WAYFINDER_MAP.md`.
- [x] Created decision tickets in `wayfinder/tickets/`:
    - `research-toml-to-loro.md`
    - `prototype-toml-loro-mapping.md`
    - `design-backend-sync-engine.md`
    - `workspace-orchestration-protocol.md`
    - `ux-error-handling-strategy.md`
- [x] Claimed ticket `prototype-toml-loro-mapping.md`.
- [x] Built a recursive mapping prototype in `sheetah-model/src/bin/proto_toml_loro.rs` that converts `toml::Value` to `LoroValue` trees for insertion into `LoroDoc`.
- [x] Resolved `research-toml-to-loro.md` via prototype implementation.
- [x] Dismissed `workspace-orchestration-protocol.md` (Loro's document/sub-doc management handles orchestration).

### In Progress
- [ ] `ux-error-handling-strategy.md`: Design how to surface/communicate errors (validation, concurrency) without disrupting the user experience.

## Not yet specified

## Out of scope

