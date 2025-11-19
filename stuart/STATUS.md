# Tantivy Fork Status

**Date:** 2025-11-19
**Branch:** nov19
**Status:** Phase 1-2 INCOMPLETE - EdgeNgramFilter wrong

**Current task:** Fix EdgeNgramFilter to apply ONLY at index time, not query time.

**Blocker:** Algolia doesn't tokenize queries to ngrams. Query "lap" searches as single term, not `["la", "lap"]`. See `ALGOLIA_QUERY_BEHAVIOR.md`.

**Fix needed:** Separate index/search analyzers in schema. Tantivy supports `set_search_analyzer()` - need to implement.

**Term format:** Clarified - 's' byte is correct type indicator. See `TERM_FORMAT_CLARIFICATION.md`.

**E2E spike results:** 
- Corruption: `"title\0sla"` instead of `"title\0la"` ❌
- Manual queries: Work correctly, return 1 hit ✅
- Conclusion: Corruption is cosmetic, doesn't break functionality

## Phase 1: EdgeNgramFilter - COMPLETE

**File:** `src/tokenizer/edge_ngram_filter.rs` (190 lines)
**Tests:** 6/6 passing
**Time:** ~1 hour

### Implementation
- `TokenFilter` trait following `SplitCompoundWords` pattern
- Generates edge ngrams per token (post-tokenization)
- UTF-8 character boundary handling
- Position preservation for phrase query compatibility

### Validation
```
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_simple ... ok
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_multi_word ... ok
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_utf8 ... ok
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_short_word ... ok
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_zero_min ... ok (panic test)
test tokenizer::edge_ngram_filter::tests::test_edge_ngram_invalid_range ... ok (panic test)
```

### Key Findings
- `NgramTokenizer` confirmed to operate on full text, not per-word
- EdgeNgramFilter correctly operates on pre-tokenized words
- "Gaming Laptop" → ["ga", "gam", "gami", "gamin", "gaming", "la", "lap", "lapt", "lapto", "laptop"]

## Next: Phase 2 - Integration Tests (3h)

**File:** `tests/edge_ngram_json_integration.rs`

Tests needed:
1. Indexing produces correct terms (verify no "title\0sla" corruption)
2. Manual TermQuery works with edge ngrams
3. QueryParser creates PhraseQuery (expected failure - documents bug)
4. Multi-word values work correctly

**Goal:** Isolate QueryParser bug before fixing it in Phase 3.

## Phase 3: QueryBehavior Enum (8h)
- Add enum to `src/schema/text_options.rs`
- Modify `src/query/query_parser/query_parser.rs`
- Fix QueryParser multi-token handling for edge ngrams

## Phase 4: Documentation (2h)
- Completion summary
- Integration guide for Flapjack
- Update this file with final results

## Validated Facts

### Upstream Tantivy
- JSON fields support custom tokenizers via `JsonObjectOptions::set_indexing_options()`
- API already public
- Test baseline: `tests/upstream_json_text_baseline.rs` (4/4 passing)

### Algolia Behavior
- Per-word prefix matching confirmed
- Query "lap" matches "Gaming Laptop" (2nd word)
- Proof: `stuart/research/algolia_test_1/critical_test_v2.js`

### Implementation Architecture
- EdgeNgramFilter as `TokenFilter` (not `Tokenizer`)
- Must use `TextAnalyzer::builder().filter()` chain
- Cannot use `NgramTokenizer` directly for per-word ngrams

## Files

**Active:**
- `src/tokenizer/edge_ngram_filter.rs` - Phase 1 implementation
- `src/tokenizer/mod.rs` - Registration
- `tests/upstream_json_text_baseline.rs` - Upstream validation

**Reference:**
- `stuart/ALGOLIA_VALIDATION.md` - Algolia behavior proof
- `stuart/IMPLEMENTATION_PLAN.md` - Phase details
- `stuart/research/algolia_test_1/` - Empirical test code

**Next:** Create `tests/edge_ngram_json_integration.rs`