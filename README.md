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
```


## Prerequisites

This tool relies on the official `sketchtool` binary. Ensure Sketch is installed on your system. The current implementation assumes `sketchtool` is located at:

`/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool`