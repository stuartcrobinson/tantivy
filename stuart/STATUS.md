# Tantivy Fork Status

**Date:** 2025-11-19
**Status:** CLEAN START - Previous fork deleted
**Branch:** None (starting fresh)
**Tests:** Upstream only

## Current State

- No active fork branch
- No EdgeNgramFilter implementation
- No QueryBehavior enum
- Starting from Tantivy upstream clean slate

## What We Know

### Validated via Tests
- Upstream JSON TEXT fields work (exact matching only)
- Multi-word tokenization works: "Gaming Laptop" → ["gaming", "laptop"]
- Prefix search correctly fails without edge ngrams (expected)
- Test: `tests/upstream_json_text_baseline.rs` - 4/4 passing

### Validated via Algolia API
- Algolia does **per-word** prefix matching
- Query "lap" matches "Gaming Laptop" (2nd word)
- Query "gam" matches "Gaming Mouse" (1st word)
- Proof: `stuart/research/algolia_test_1/critical_test_v2.js`
- **Implication:** Must use EdgeNgramFilter (post-tokenization), NOT NgramTokenizer (full-text)

## Previous Attempt (DELETED)

Branch `custom/tokenizer` existed, claimed 943/943 tests passing, but:
- Had "weird errors" (see flapjack historical docs)
- Compilation failures documented in `SESSION_2025_11_19.md`
- JSON text tokenization bug: terms corrupted as `"title\0sla"` instead of `"title\0la"`
- Bug exists in **upstream Tantivy** (not fork-introduced)
- Root cause: `set_type()` adds 's' byte before tokenization, gets preserved in truncation

**Key learning:** Term structure confusion between test/query encoding vs production indexing encoding was never resolved.

## Implementation Plan

See `IMPLEMENTATION_PLAN.md` for phase breakdown:
1. EdgeNgramFilter (6h)
2. Integration tests (3h) 
3. QueryBehavior enum (8h)
4. Documentation (2h)

**Total:** 19h estimated

## Next Steps

1. Implement EdgeNgramFilter following `SplitCompoundWords` pattern
2. Write unit tests BEFORE integration
3. Validate indexing produces correct terms
4. Add QueryBehavior to fix QueryParser multi-token handling
5. Update docs incrementally (not separate phase)

## References

- Algolia validation: `stuart/research/algolia_test_1/`
- Upstream baseline: `tests/upstream_json_text_baseline.rs`
- Implementation plan: `IMPLEMENTATION_PLAN.md`
- Previous attempt analysis: See flapjack repo historical docs