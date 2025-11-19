# Phase 3: QueryBehavior Implementation - Status

**Date:** 2025-11-19
**Status:** READY - Implementing QueryBehavior
**Prerequisite:** EdgeNgramFilter complete ✅, term format understood ✅

**Cleared blocker:** 's' byte is not corruption - it's the correct Type::Str indicator. See `TERM_FORMAT_CLARIFICATION.md`.

## What's Working

✅ EdgeNgramFilter generates correct ngrams: `["la", "lap", "lapt", "lapto", "laptop"]`
✅ Terms indexed (with cosmetic corruption): `"title\0sla"`, `"title\0slap"`, etc.
✅ Manual TermQuery matches: 1 hit on "lap" query
✅ Multi-word values work: Both "gam" and "lap" match "Gaming Laptop"

## The Problem

QueryParser creates PhraseQuery for multi-token inputs:
```rust
// User query: "lap"
// EdgeNgram tokenizer produces: ["la", "lap"] at SAME position
// QueryParser creates: PhraseQuery with offsets [(0, "la"), (0, "lap")]
// In "Gaming Laptop": Both indexed at position 1
// PhraseQuery matches because relative offsets align
// Result: 1 hit (accidentally works!)
```

**But this breaks with different query patterns:**
```rust
// If QueryParser used offsets [(0, "la"), (1, "lap")]:
// Would require consecutive positions → 0 hits
```

**Real issue:** EdgeNgramFilter assigns same position to all ngrams from one word. PhraseQuery accidentally works because query and index have matching relative positions. Fragile - any change to QueryParser offset logic breaks it.

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

- [x] Query "lap" returns 1 hit on "Gaming Laptop" (already works via PhraseQuery accident)
- [x] Query "gam" returns 1 hit on "Gaming Laptop" (already works)
- [ ] QueryBehavior::TermsOr explicitly uses BooleanQuery (not relying on position accident)
- [ ] Phrase queries still work on non-edge-ngram fields
- [ ] cargo test passes (943+ tests)
- [ ] No regressions in query_parser tests

## Updated Understanding (2025-11-19)

**Surprising finding:** Edge ngram queries already work via accidental PhraseQuery match.

**Why implement QueryBehavior anyway?**
1. Current behavior is fragile (relies on position bug)
2. Explicit OR semantics clearer than accidental phrase match
3. Enables proper phrase query support if positions get fixed later
4. Algolia-compatible behavior (treats prefix as OR of ngrams, not phrase)