# webtk

`webtk` is a command-line tool designed to interact with and manage assets from design files, starting with Sketch files.

## Prerequisites

This tool relies on the official `sketchtool` binary. Ensure Sketch is installed on your system. The current implementation assumes `sketchtool` is located at:

`/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool`

## Usage

Use the `--help` flag for detailed command information:

```bash
webtk --help
```

### Sketch Commands

The `sketch` subcommand handles Sketch file operations.

#### List Artboards

Lists the name and unique ID of all artboards (and symbols) within a specified Sketch file.

```bash
webtk sketch list-artboards tests/data/sample-sketch.sketch
```
