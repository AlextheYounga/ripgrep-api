# Implementation Plan - ripgrep API wrapper

## Approach
- Provide a user-friendly Rust API with rg semantics and vocabulary.
- Build on ripgrep component libraries (grep-* + ignore + globset).
- Defaults: smart case, ignore rules on, hidden off, binary detection on.
- Reference docs/ripgrep.md for parity and behavior details.

## Phases

### Phase 0 - Scope and foundations
- Goals: streaming search, structured match data, convenience helpers.
- Non-goals (v1): no CLI binary, no arg parsing, no printer formats beyond structured matches.
- Dependencies: ignore, globset, grep-searcher, grep-regex, grep-matcher.
- Optional features: pcre2 flag, encoding support if needed.
- Confirm default semantics and vocabulary mapping.

### Phase 1 - Core API and skeleton engine
- Public API: SearchBuilder::new(pattern) or rg(pattern).
- Core types: Match, SubMatch, ContextLine, SearchError.
- Module layout: builder, config, engine, matcher, search, types, error.
- Execution flow (basic): WalkBuilder + matcher + SearcherBuilder + Sink.
- Tests: basic regex search, build() errors, iterator correctness.

### Phase 2 - rg semantics parity
- Smart case, ignore rules, hidden/follow toggles, binary detection.
- Glob and file type filters, max depth, max file size.
- Context lines (before/after/context), limits, files-with-matches, count.
- Tests: ignore handling, smart-case, glob/type, context/limit behavior.

### Phase 3 - Non-CLI features and docs
- Custom sinks or callbacks for streaming processing.
- In-memory APIs: search_slice, search_reader.
- Programmatic file type definitions and overrides.
- Documentation: README examples, "rg flag -> API method" table, rustdoc examples.
- Tests: in-memory search, custom sink behavior.

### Phase 4 - Optional features and tuning
- pcre2 feature integration and tests.
- Performance tuning (buffer sizes, mmap, threads) and benchmarks.
- Additional semantic edge cases from rg tests/docs.
