# Phase 3: QueryBehavior Implementation - Status

**Date:** 2025-11-19
**Status:** Ready to implement
**Prerequisite:** EdgeNgramFilter complete, tests passing

## What's Working

✅ EdgeNgramFilter generates correct ngrams: `["la", "lap", "lapt", "lapto", "laptop"]`
✅ Terms indexed (with cosmetic corruption): `"title\0sla"`, `"title\0slap"`, etc.
✅ Manual TermQuery matches: 1 hit on "lap" query
✅ Multi-word values work: Both "gam" and "lap" match "Gaming Laptop"

## The Problem

QueryParser creates PhraseQuery for multi-token inputs:
```rust
// User query: "lap"
// EdgeNgram tokenizer produces: ["la", "lap"]
// QueryParser creates: PhraseQuery(["la", "lap"]) requiring consecutive match
// In "Gaming Laptop": "la" and "lap" exist but NOT consecutively
// Result: 0 hits
```

## The Solution

Add schema-level QueryBehavior enum to control multi-token query construction:

```rust
pub enum QueryBehavior {
    Phrase,    // Default: consecutive terms (existing behavior)
    TermsOr,   // Edge ngrams: OR'd term queries
}
```

When field uses `QueryBehavior::TermsOr`, QueryParser creates:
```rust
BooleanQuery {
    subqueries: [
        (Should, TermQuery("la")),
        (Should, TermQuery("lap"))
    ]
}
```

## Implementation Checklist

- [ ] Add enum to `src/schema/text_options.rs`
- [ ] Extend `TextFieldIndexing` with getter/setter
- [ ] Modify `src/query/query_parser/query_parser.rs`:
  - [ ] Text field path (~line 960)
  - [ ] JSON field path (~line 1040)
- [ ] Schema serialization (skip_serializing_if default)
- [ ] Update existing integration test to use TermsOr
- [ ] Verify all upstream tests still pass

## Files to Modify

1. `src/schema/text_options.rs` - Add QueryBehavior enum
2. `src/query/query_parser/query_parser.rs` - Check behavior before creating PhraseQuery
3. `tests/edge_ngram_e2e_spike.rs` - Add QueryBehavior to schema config

## Acceptance Criteria

- [ ] Query "lap" returns 1 hit on "Gaming Laptop"
- [ ] Query "gam" returns 1 hit on "Gaming Laptop"
- [ ] Phrase queries still work on non-edge-ngram fields
- [ ] cargo test passes (943+ tests)
- [ ] No regressions in query_parser tests