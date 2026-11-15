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
- API Ergonomics: Use `LoroDoc::get_map` and `insert_container` for nested structures. Import/export mechanism is essential for syncing replicas.

## Decisions so far

- [Target Pattern: Mutable-to-Diff](wayfinder/tickets/grilling-implementation-pain-points.md): The CRDT engine must support a workflow where arbitrary changes to a mutable JSON-like view are automatically translated into CRDT diffs.
- [Sync Engine Architecture](wayfinder/tickets/design-backend-sync-engine.md): An "Orchestrated Change Stream" (OCS) using a sharded worker model to ensure document-level sequencing and authoritative validation.
- [Mapping Strategy](wayfinder/tickets/prototype-toml-loro-mapping.md): Recursive mapping of structured data (like TOML) into `LoroValue` trees to bridge external formats and CRDT state.
- [CRDT Engine Selection](wayfinder/tickets/research-loro-vs-yrs-ergonomics.md): Loro is chosen due to its superior mapping and sync ergonomics as demonstrated in MWEs.
- [Error Handling Strategy](wayfinder/tickets/ux-error-handling-strategy.md): Validation errors are treated as first-class, synchronized data within the `LoroDoc` using a specialized `_validation_errors` key.

## Not yet specified

## Out of scope

- [YRS Integration Prototypes](wayfinder/tickets/prototype-yrs-minimal-integration.md): Ruled out in favor of Loro.
- [Alternative CRDT Research](wayfinder/tickets/research-alternative-crdt-crates.md): Ruled out in favor of Loro.
- [CRDT Migration Assessment](wayfinder/tickets/task-switching-cost-assessment.md): Ruled out as Loro is the chosen engine.
- [Workspace Orchestration Protocol](wayfinder/tickets/workspace-orchestration-protocol.md): Loro's native document/sub-doc management handles orchestration requirements.
