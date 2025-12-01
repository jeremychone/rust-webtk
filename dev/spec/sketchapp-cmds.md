# Sketchtool Commands Used

This document outlines the `sketchtool` commands executed by `sketchdev`.

The `sketchtool` binary path is `/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool`.

## 1. List Artboards

Used in `src/sketch-doc.ts` to retrieve a list of artboards and symbols from a Sketch file.

````bash
sketchtool --include-symbols=YES list artboards <sketch_file>
````

Arguments:
- `--include-symbols=YES`: Includes symbols in the list output.
- `list artboards`: The command to list artboards.
- `<sketch_file>`: The path to the input Sketch file.

## 2. Export Artboards

Used in `src/export-image.ts` to export artboards (and symbols) to image formats (SVG, PNG, JPEG).

````bash
sketchtool \
  --format=<format> \
  --include-symbols=YES \
  [--items=<id1,id2,...>] \
  --output=<output_directory> \
  export artboards <sketch_file>
````

Arguments:
- `--format=<format>`: Specifies the export format (e.g., `svg`, `png`, `jpeg`).
- `--include-symbols=YES`: Includes symbols in the export process.
- `[--items=<id1,id2,...>]`: Optional. Specifies a comma-separated list of Artboard or Symbol IDs to export. If omitted, all matching artboards/symbols are exported.
- `--output=<output_directory>`: Specifies the directory where exported assets will be placed.
- `export artboards`: The command to export artboards.
- `<sketch_file>`: The path to the input Sketch file.
