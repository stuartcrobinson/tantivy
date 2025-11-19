# Test Session Findings - 2025-11-19

## Tests Written

1. `tests/verify_query_term_bytes.rs` - Term byte comparison
2. `tests/queryparser_edge_ngram_behavior.rs` - PhraseQuery behavior
3. `tests/compare_term_bytes.rs` - Indexed vs query term structure
4. `tests/check_token_positions.rs` - EdgeNgramFilter position output
5. `tests/phrase_query_position_debug.rs` - Position offset matching

## Key Discoveries

### 1. 's' Corruption is Symmetric
**Test:** `compare_term_bytes.rs`
- Indexed: `"title\0slap"`
- Query: `"\0\0\0\0jtitle\0slap"`
- Both have 0x73 ('s') after path terminator
- Term lookup returns FOUND

**Conclusion:** Corruption doesn't break queries because both sides add 's' identically.

### 2. EdgeNgramFilter Assigns Same Position to All Ngrams
**Test:** `check_token_positions.rs`
```
"Gaming" → ["ga":pos0, "gam":pos0, "gami":pos0, "gamin":pos0, "gaming":pos0]
"Laptop" → ["la":pos1, "lap":pos1, "lapt":pos1, "lapto":pos1, "laptop":pos1]
```

**Expected (for phrase query correctness):**
```
"Gaming" → ["ga":pos0, "gam":pos1, "gami":pos2, "gamin":pos3, "gaming":pos4]
"Laptop" → ["la":pos5, "lap":pos6, ...]
```

**Impact:** PhraseQuery accidentally matches because query tokenization also assigns same position to ngrams.

### 3. PhraseQuery Matches on Same-Position Terms
**Test:** `phrase_query_position_debug.rs`
- Query with offsets `[(0, "la"), (1, "lap")]` → 0 hits (expects consecutive)
- Query with offsets `[(0, "la"), (0, "lap")]` → 1 hit (both at position 1)

**Mechanism:** QueryParser tokenizes "lap" → `["la", "lap"]` at same position, constructs PhraseQuery with relative offsets `[(0, "la"), (0, "lap")]`. This matches indexed terms both at position 1.

### 4. QueryParser Creates PhraseQuery (Not TermQuery)
**Test:** `verify_query_term_bytes.rs`
```
Query structure: PhraseQuery { 
  phrase_terms: [(0, Term(..., "la")), (0, Term(..., "lap"))], 
  slop: 0 
}
Hits: 1
```

**Confirms:** QueryParser uses PhraseQuery for multi-token queries, but accidentally works due to position bug.

## Implications for Phase 3

**Original assumption:** QueryParser creates PhraseQuery requiring consecutive terms → 0 hits.
**Reality:** QueryParser creates PhraseQuery with same-position offsets → 1 hit (accidental success).

**Why implement QueryBehavior despite this?**
1. Behavior is fragile - relies on position bug
2. If EdgeNgramFilter position logic changes (e.g., fix to increment), queries break
3. Explicit BooleanQuery with OR semantics is correct approach
4. Algolia compatibility requires OR semantics, not phrase matching

**Decision:** Proceed with Phase 3 implementation. QueryBehavior::TermsOr provides:
- Explicit OR semantics (not accidental phrase match)
- Robustness against future position fixes
- Correct semantic model for prefix search

## Test Files to Keep

- `tests/edge_ngram_e2e_spike.rs` - Original working test
- `tests/compare_term_bytes.rs` - Documents 's' corruption
- `tests/check_token_positions.rs` - Documents position bug
- `tests/phrase_query_position_debug.rs` - Proves PhraseQuery behavior

**Others can be deleted** after Phase 3 ships (served diagnostic purpose).

## Next Action

Implement Phase 3: QueryBehavior enum + QueryParser modification.

Corruption and position bugs are documented, understood, and non-blocking.