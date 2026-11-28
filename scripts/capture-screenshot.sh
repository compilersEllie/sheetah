#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
workspace_root=$(cd -- "$script_dir/.." && pwd)
cd "$workspace_root"

usage() {
    cat <<'USAGE'
Usage: scripts/capture-screenshot.sh [NAME] [OUTPUT_DIR] [--keep-db]

Capture a Sheetah screenshot with the headless GPUI renderer.

  NAME        PNG filename (default: sheetah.png)
  OUTPUT_DIR  Destination directory (default: docs/screenshots)
  --keep-db   Keep the existing sheetah-gpui/sheetah.sqlite instead of
              deleting it to seed a fresh Sheet1 grid.

Examples:
  scripts/capture-screenshot.sh multi-table.png
  scripts/capture-screenshot.sh custom.png /tmp/screenshots --keep-db
USAGE
}

name="sheetah.png"
output_dir="$workspace_root/docs/screenshots"
keep_db=0
positional=0

for argument in "$@"; do
    case "$argument" in
        -h|--help)
            usage
            exit 0
            ;;
        --keep-db)
            keep_db=1
            ;;
        -*)
            echo "error: unknown option: $argument" >&2
            usage >&2
            exit 1
            ;;
        *)
            if [[ $positional -eq 0 ]]; then
                name=$argument
            elif [[ $positional -eq 1 ]]; then
                output_dir=$argument
            else
                echo "error: too many arguments" >&2
                usage >&2
                exit 1
            fi
            positional=$((positional + 1))
            ;;
    esac
done

mkdir -p "$output_dir"
output_dir=$(cd -- "$output_dir" && pwd)

if [[ $keep_db -eq 0 ]]; then
    rm -f "$workspace_root/sheetah-gpui/sheetah.sqlite"
fi

SHEETAH_SCREENSHOT_DIR="$output_dir" \
SHEETAH_SCREENSHOT_NAME="$name" \
    cargo test -p sheetah-gpui --locked --quiet capture_screenshot_when_requested -- --nocapture

output="$output_dir/$name"
if [[ ! -f "$output" ]]; then
    echo "error: expected screenshot was not created: $output" >&2
    exit 1
fi

echo "Saved screenshot: $output"
