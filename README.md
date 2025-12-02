# webtk

`webtk` is a command-line tool designed to interact with and manage assets from design files, starting with Sketch files.

See [Prerequisites](#prerequisites) (in short, have sketchapp installed which should have sketchtool as well)

## Usage

Use the `--help` flag for detailed command information:

```sh
webtk --help
```

### Sketch Commands

The `sketch` subcommand handles Sketch file operations.

```sh
# List all of the Artboards
webtk sketch list-artboards tests/data/sample-sketch.sketch

# List all Artboards with glob
webtk sketch list-artboards -g "ico/*" tests/data/sample-sketch.sketch 

# Export all ico/ artboards
webtk sketch export -g "ico/*" --format "svg,png" -o ".out/icons" tests/data/sample-sketch.sketch 

# Export as SVG symbols (all icons combined into one SVG with <symbol> elements)
webtk sketch export -g "ico/*" --format "svg-symbols" -o ".out/icons/symbols.svg" tests/data/sample-sketch.sketch 

```

- `-g` is a glob on the artboard name. For mulitple globs do `-g "ico/*" -g "logo/*`
- `--format` is the format of the export. Can be `svg`, `png`, `jpeg`, `svg-symbols`. 
    - For multiple, either comma delimited `--format "svg,png` or multiple `--format svg --format png`
    - `svg-symbols` exports all matched artboards as SVG `<symbol>` elements in a single SVG file

## Prerequisites

This tool relies on the official `sketchtool` binary. Ensure Sketch is installed on your system. The current implementation assumes `sketchtool` is located at:

`/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool`