# Screenshots

These images are captured from the real GPUI renderer in headless mode, so they show the same
layout and styling as the desktop app without needing a display server.

## Current UI

![Multi-table spreadsheet with the table sidebar](screenshots/multi-table.png)

The sidebar lists persisted tables, the grid shows the active table, and the title bar uses the
client-side decoration layout.

## Original spreadsheet MVP

![Single-table spreadsheet MVP](screenshots/mvp.png)

This is the UI from PR #19: one editable 4x4 grid with cell editing and window controls.

## Regenerating a screenshot

The capture harness is an opt-in test named `capture_screenshot_when_requested` in
`sheetah-gpui/src/main.rs`. It does nothing unless `SHEETAH_SCREENSHOT_DIR` is set, so it is safe to
leave in the normal test suite.

From the workspace root:

```bash
rm -f sheetah-gpui/sheetah.sqlite
mkdir -p docs/screenshots
SHEETAH_SCREENSHOT_DIR="$PWD/docs/screenshots" \
SHEETAH_SCREENSHOT_NAME=multi-table.png \
  cargo test -p sheetah-gpui --offline --quiet capture_screenshot_when_requested -- --nocapture
```

- `SHEETAH_SCREENSHOT_DIR` enables capture and selects the output directory.
- `SHEETAH_SCREENSHOT_NAME` sets the PNG filename (default: `sheetah.png`).
- Removing `sheetah-gpui/sheetah.sqlite` first makes the app seed a fresh `Sheet1` 4x4 grid, so the
  screenshot is reproducible.
- The harness uses `HeadlessAppContext` with the platform headless renderer, loads the app fonts,
  opens a 900x560 client-decorated window, waits for startup to finish, then calls
  `capture_screenshot` and saves the PNG. At 2x scaling the output is 1800x1120.
- Because the test runs in `sheetah-gpui/`, the database it creates is
  `sheetah-gpui/sheetah.sqlite`; the output directory is the absolute path you supplied.

Use the same command with a different `SHEETAH_SCREENSHOT_NAME` for additional views. To capture a
different UI state, seed the database before running the test (for example, by creating tables in a
separate app run) instead of deleting it.
