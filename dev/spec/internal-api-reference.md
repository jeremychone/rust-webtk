# webtk - Internal API Reference

This document lists public types and function signatures exposed by internal modules, designed for LLM/AI reference.

## Core Types and Error Handling

(Defined in `src/error.rs` and re-exported in `src/main.rs`)

```rust
use derive_more::{Display, From};
use simple_fs::SPath;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Display, From)]
pub enum Error {
    Custom(String),
    FileNotFound(SPath),
    SerdeJson(serde_json::Error),
    // ... other variants
}
```

## Service: Sketch (`service::sketch`)

(Defined in `src/service/sketch/mod.rs` and submodules)

### Artboard

Represents a Sketch artboard identified by name and UID.

```rust
#[derive(Debug, Clone)]
pub struct Artboard {
    pub name: String,
    pub uid: String,
}
```

### Functions

```rust
use simple_fs::SPath;

// from sketch_list.rs
pub fn list_artboards(
    sketch_file: impl AsRef<SPath>, 
    glob_patterns: Option<&[&str]>
) -> Result<Vec<Artboard>>;

// from sketch_export.rs
pub fn export_artboards(
    sketch_file: impl AsRef<SPath>,
    glob_patterns: Option<&[&str]>,
    formats: &[&str],
    output_dir: impl AsRef<SPath>,
    flatten: bool,
) -> Result<Vec<String>>;
```

## Support Utilities (`support`)

(Defined in `src/support/mod.rs` and submodules)

### support::files

Utilities for file system checks and safe deletion.

```rust
use simple_fs::SPath;

pub fn check_file_exists(path: &SPath) -> Result<()>;
pub fn looks_like_file_path(path: &SPath) -> bool;
pub fn safer_delete_dir(dir_path: &SPath) -> Result<bool>;
```

### support::globs

Utilities for handling glob pattern matching.

```rust
use globset::GlobSet;

pub fn build_glob_set(patterns: Option<&[&str]>) -> Result<Option<GlobSet>>;
pub fn matches_glob_set(glob_set: Option<&GlobSet>, value: &str) -> bool;
```

### support::strings

String manipulation utilities.

```rust
pub fn canonicalize_name(name: &str) -> String;
```

### support::xmls

XML processing utilities using `xmltree`.

```rust
use xmltree::XMLNode;

pub fn extract_root_attribute(xml_content: &str, attr_name: &str) -> Option<String>;
pub fn extract_root_inner_nodes(xml_content: &str) -> Option<Vec<XMLNode>>;
pub fn transform_nodes_id_attributes<F>(nodes: Vec<XMLNode>, transform_fn: F) -> Vec<XMLNode>
where F: Fn(&str) -> String;
pub fn nodes_to_string(nodes: &[XMLNode]) -> String;
```
