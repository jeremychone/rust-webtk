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
        "1F58B244-44D6-4809-8DBC-F82F5B5F385C" : {
          "name" : "ico\/check-circle-ot"
        },
        "3D2807C2-5D15-4248-A81B-9FD6C012EFEF" : {
          "name" : "ico\/folder"
        },
        "7DCCDBD2-A14E-429F-AF70-9BF4B55BEAC0" : {
          "name" : "ico\/media"
        },
        "8C5BC476-7D87-4774-8455-39A251DCDF68" : {
          "name" : "ico\/chevron-right"
        },
        "9A28AF57-B6BE-4D9F-B1BD-B49BE762BC2D" : {
          "name" : "ico\/gear"
        },
        "45BD9E9E-EEEF-4EA3-A802-1BC2BD442B40" : {
          "name" : "ico\/place\/out"
        },
        "81A68DFE-9D5D-4EDF-BA45-96792281D7BA" : {
          "name" : "ico\/chevron-down"
        },
        "103E09A8-54E3-4071-BC21-BEFC7E5F6DCA" : {
          "name" : "ico\/user\/ot"
        },
        "725A22C5-182F-4FCE-93F3-6C1B2EE68452" : {
          "name" : "ico\/upload"
        },
        "76807D4A-7919-4B82-9EDC-D21AC68FF324" : {
          "name" : "ico\/lock-ot"
        },
        "666099E1-9733-4CCE-BFE3-F3C737FC8E9A" : {
          "name" : "ico\/tenant"
        },
        "D1183DEA-8435-4687-9C04-082CE6C77CF0" : {
          "name" : "ico\/search"
        },
        "F16A8474-EB77-47CA-BAFF-0071C5B76E13" : {
          "name" : "ico\/process-circle-ot"
        }
      },
      "name" : "ico\/"
    },
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

Arguments:
- `--format=<format>`: Specifies the export format (e.g., `svg`, `png`, `jpeg`).
- `--include-symbols=YES`: Includes symbols in the export process.
- `[--items=<id1,id2,...>]`: Optional. Specifies a comma-separated list of Artboard or Symbol IDs to export. If omitted, all matching artboards/symbols are exported.
- `--output=<output_directory>`: Specifies the directory where exported assets will be placed.
- `export artboards`: The command to export artboards.
- `<sketch_file>`: The path to the input Sketch file.
