# WAYFINDER MAP: CRDT Engine Selection

## Destination

A decision on the core CRDT engine for `@sheetah-model/` (evaluating `loro`, `yrs`, or other viable alternatives), accompanied by a cost/benefit tradeoff summary and a minimal working example (MWE) demonstrating the chosen engine's integration patterns.

## Notes

- Focus on ergonomics and "accessibility" of the API to reduce implementation pain.
- Evaluate switching costs relative to current `loro` progress.
- Documentation should be in `.md` files.

## Decisions so far

- [Target Pattern: Mutable-to-Diff](wayfinder/tickets/grilling-implementation-pain-points.md): The CRDT engine must support a workflow where arbitrary changes to a mutable JSON-like view are automatically translated into CRDT diffs.
## Not yet specified

- Investigating the viability of `yrs` or other alternatives due to implementation friction with `loro`.

## Out of scope

