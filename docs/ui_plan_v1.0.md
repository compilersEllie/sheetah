# Wayfinder Specification
*Final agreed-upon specs for UI/DSL implementation (v1.0)*

 ---

## 1. Backend Navigation & Tagging
### Structure
```

Project → Backend (e.g., "Finance") → Table (e.g., "Budget") → Views (e.g., "Q3 Summary")

```

### Tagging System (Obsidian-style)
 - **Syntax**: `#tagname` (e.g., `#sales`, `#finance`)
 - **Search**:
   - `#sales` returns *all views tagged with "sales"* (bypasses tree navigation)
   - Tags appear in sidebar filters as `#marketing`, `#finance`
 - **Acceptance**:
   > Search `#sales` → shows all tagged views directly; *no hierarchical traversal required*

 ---

## 2. DSL Syntax & Editing
### Core Syntax
 ```chart table.sum("Sales")```

- Purpose: Visualize data (default in markdown cells)

### DSL Types

┌─────────────┬───────────────────────────────────────────┬──────────────────────────────┐
│ Type        │ Example                                   │ Usage Context                │
├─────────────┼───────────────────────────────────────────┼──────────────────────────────┤
│ Chart       │ chart table.avg("Profit")                 │ Markdown cells, view configs │
├─────────────┼───────────────────────────────────────────┼──────────────────────────────┤
│ Filter      │ filter table.where("Region", "EU")        │ View configuration           │
├─────────────┼───────────────────────────────────────────┼──────────────────────────────┤
│ Calculation │ calc table.add("Profit", "Revenue * 0.2") │ View configuration           │
└─────────────┴───────────────────────────────────────────┴──────────────────────────────┘

### Editing Rules

- Markdown cells:
   - ```chart ... ``` syntax shown as code (highlighted)

- View configuration:
   - "Edit" button opens real-time editor for ```chart ... ```
   - Changes update view immediately (no save required)

────────────────────────────────────────────────────────────────────────────────

3. Final Validation

✅ Tags: #tagname (Obsidian syntax) — not tag:tagname
✅ DSL: ```chart ... ``` — **not** <!-- DSL: ... -->
✅ Editable: DSL is fully editable during view configuration (not read-only)
✅ Priority Order:
1. Backend tree + tags
2. DSL syntax/editing
3. Independent panes
4. Global search (content + markdown)
5. Dark mode default

### Layout

Left side:
  Top: File&Backend tree
  Bottom: Task management (top tasks, new task entry), expands to Kanban view

Centre:
  Panes in flexible customizable layout
    Each pane showing a different table/note/chart

Right side:
  Today's note OR daily note OR Calendar
