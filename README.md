# ripgrep-api

Coder-friendly Rust API wrapper around ripgrep's core crates.

## Install

Crates.io (once published):

```toml
ripgrep-api = "0.1"
```

Git dependency (current repo):

```toml
ripgrep-api = { git = "https://github.com/AlextheYounga/ripgrep-api.git" }
```

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

## PCRE2 (feature flag)

```rust
# #[cfg(feature = "pcre2")]
# {
use ripgrep_api::SearchBuilder;

let matches = SearchBuilder::new(r"(foo)(bar)\1")
    .pcre2()
    .search_slice(b"foobarfoo")?;

assert_eq!(matches.len(), 1);
# Ok::<(), ripgrep_api::SearchError>(())
# }
```

Enable the feature in Cargo:

```toml
ripgrep-api = { version = "0.1", features = ["pcre2"] }
```

## Performance knobs

```rust
use ripgrep_api::SearchBuilder;
use grep_searcher::MmapChoice;

let matches = SearchBuilder::new("alpha")
    .path(".")
    .threads(4)
    .memory_map(unsafe { MmapChoice::auto() })
    .heap_limit(64 * 1024)
    .build()?
    .collect::<Vec<_>>();
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
| `-P/--pcre2` | `pcre2()` (feature: `pcre2`) |
| `-j/--threads` | `threads(...)` |
| `--mmap` | `memory_map(...)` |
| `--no-mmap` | `memory_map(MmapChoice::never())` |
| `--heap-limit` | `heap_limit(...)` |
