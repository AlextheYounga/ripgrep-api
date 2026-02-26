# ripgrep-api

Coder-friendly Rust API wrapper around ripgrep's core crates.

## Quickstart

```rust
use ripgrep_api::SearchBuilder;

let matches: Vec<_> = SearchBuilder::new("todo")
    .path(".")
    .glob("**/*.rs")
    .smart_case()
    .context(2)
    .build()?
    .collect();

for mat in matches {
    println!("{}:{}:{}", mat.path.display(), mat.line.unwrap_or(0), mat.line_text);
}
# Ok::<(), ripgrep_api::SearchError>(())
```

## Streaming callbacks

```rust
use ripgrep_api::{Match, SearchBuilder};

let root = ".";
SearchBuilder::new("alpha")
    .path(root)
    .for_each(|mat: &Match| {
        println!("{}:{}", mat.path.display(), mat.line.unwrap_or(0));
        true
    })?;
# Ok::<(), ripgrep_api::SearchError>(())
```

## In-memory search

```rust
use ripgrep_api::SearchBuilder;

let haystack = b"zero\nmatch\nthree\n";
let matches = SearchBuilder::new("match")
    .search_slice(haystack)?;

assert_eq!(matches.len(), 1);
# Ok::<(), ripgrep_api::SearchError>(())
```

## Custom file types and overrides

```rust
use ripgrep_api::SearchBuilder;

let matches = SearchBuilder::new("alpha")
    .path(".")
    .type_add("notes", "*.note")
    .type_("notes")
    .glob("!ignored.note")
    .build()?
    .collect::<Vec<_>>();
# Ok::<(), ripgrep_api::SearchError>(())
```

## rg flag -> API method

| rg flag | API method |
| --- | --- |
| `-g/--glob` | `glob(...)` |
| `-t/--type` | `type_(...)` |
| `-T/--type-not` | `type_not(...)` |
| `-d/--max-depth` | `max_depth(...)` |
| `--max-filesize` | `max_filesize(...)` |
| `-i/--ignore-case` | `ignore_case()` |
| `-S/--smart-case` | `smart_case()` |
| `-w/--word-regexp` | `word()` |
| `-x/--line-regexp` | `line_regexp()` |
| `-A/--after-context` | `after_context(...)` |
| `-B/--before-context` | `before_context(...)` |
| `-C/--context` | `context(...)` |
| `-m/--max-count` | `max_count(...)` |
| `-uuu` | `hidden()` + `ignore(false)` |
