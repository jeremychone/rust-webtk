# Sketchtool Commands Used

This document outlines the `sketchtool` commands executed by `sketchdev`.

The `sketchtool` binary path is `/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool`.

## 1. List Artboards from metadata

To list the arboard, we do 

```bash 
/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool  metadata tests/.data/sample-sketch.sketch 
```

Which will give something like: 

```
{
  "app" : "com.bohemiancoding.sketch3",
  "appVersion" : "100.3",
  "autosaved" : 0,
  "build" : 180165,
  "coeditCompatibilityVersion" : 145,
  "commit" : "2a12798812d24f66ac907f64fcd4d40afe1e1cce",
  "compatibilityVersion" : 99,
  "created" : {
    "app" : "com.bohemiancoding.sketch3",
    "appVersion" : "100.3",
    "build" : 180165,
    "coeditCompatibilityVersion" : 145,
    "commit" : "2a12798812d24f66ac907f64fcd4d40afe1e1cce",
    "compatibilityVersion" : 99,
    "variant" : "NONAPPSTORE",
    "version" : 146
  },
  "pagesAndArtboards" : {
    "50DE3142-2CB4-4557-8236-444264C60921" : {
      "artboards" : {
        "0D2D5069-35B2-4507-BEAD-1898B7B4668B" : {
          "name" : "ico\/user\/fill"
        },
        "0E3E4A2C-4613-4AB8-B186-2F6290027F37" : {
          "name" : "ico\/place\/in"
        },
        ...
      },
      "name" : "ico\/"
    },
    ...
...
```

## 2. Export Artboards

Used in `src/export-image.ts` to export artboards (and symbols) to image formats (SVG, PNG, JPEG).

```bash
sketchtool \
  --format=<format> \
  --include-symbols=YES \
  [--items=<id1,id2,...>] \
  --output=<output_directory> \
  export artboards <sketch_file>
```

for example
```sh
/Applications/Sketch.app/Contents/Resources/sketchtool/bin/sketchtool \
  --format=svg \
  --include-symbols=YES \
  --items=0D2D5069-35B2-4507-BEAD-1898B7B4668B,0E3E4A2C-4613-4AB8-B186-2F6290027F37 \
  --output=.out/ \
  export artboards tests/.data/sample-sketch.sketch
```  

Arguments:
- `--format=<format>`: Specifies the export format (e.g., `svg`, `png`, `jpeg`).
- `--include-symbols=YES`: Includes symbols in the export process.
- `[--items=<id1,id2,...>]`: Optional. Specifies a comma-separated list of Artboard or Symbol IDs to export. If omitted, all matching artboards/symbols are exported.
- `--output=<output_directory>`: Specifies the directory where exported assets will be placed.
- `export artboards`: The command to export artboards.
- `<sketch_file>`: The path to the input Sketch file.
