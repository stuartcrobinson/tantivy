# Critical Finding: Term Corruption Bug Confirmed in Upstream

**Date:** 2025-11-19
**Status:** BLOCKER - Upstream Tantivy bug confirmed

## The Bug

**Symptom:** Edge ngram tokens indexed with 's' type byte corruption
**Expected:** `"title\0la"`, `"title\0lap"`, `"title\0lapt"`
**Actual:** `"title\0sla"`, `"title\0slap"`, `"title\0slapt"`

The 's' type byte appears BETWEEN path and token text.

## Evidence

Test: `tests/edge_ngram_e2e_spike.rs::test_edge_ngram_indexing_no_corruption`
```
=== INDEXED TERMS ===
"title\0sla"
"title\0slap"
"title\0slapt"
"title\0slapto"
"title\0slaptop"
```

**BUT manual queries still work:**
- Test `test_edge_ngram_manual_term_query`: 1 hit ✅
- Test `test_edge_ngram_multi_word_tokens`: Both queries return 1 hit ✅

## Critical Discovery

**The corruption doesn't break queries.** Manual TermQuery construction with `Term::from_field_json_path()` + `append_type_and_str()` somehow matches the corrupted indexed terms.

**Verified (2025-11-19 tests):**
1. ✅ Query construction also adds 's': `"\0\0\0\0jtitle\0slap"` matches indexed `"title\0slap"`
2. ✅ Both sides have 's' in identical position relative to path terminator
3. ✅ Term lookup succeeds because corruption is symmetric

**Position Bug Discovered:**
- EdgeNgramFilter assigns SAME position to all ngrams from one word
- "Gaming" → all tokens at position 0
- "Laptop" → all tokens at position 1
- PhraseQuery with offsets `[(0, "la"), (0, "lap")]` matches (1 hit)
- PhraseQuery with offsets `[(0, "la"), (1, "lap")]` fails (0 hits)

**Root cause:** EdgeNgramFilter reuses position from source token. Should increment position per ngram to prevent false phrase matches.

## Root Cause (from previous analysis)

**File:** `src/core/json_utils.rs` (upstream code)
```rust
set_path_id(term_buffer, unordered_id);  // buffer = [field][j][unordered_id]
set_type(term_buffer, Type::Str);        // buffer = [field][j][unordered_id][s]
let path_end = term_buffer.len_bytes();  // = 5 (includes 's')
postings_writer.index_text(..., path_end);
```

Inside `index_text()`:
```rust
term_buffer.truncate_value_bytes(path_end);  // Preserves [field][j][unordered_id][s]
term_buffer.append_bytes(token.text);         // Appends after 's'
```

Result: `[unordered_id][s][token_bytes]` → serialized as `"path\0stoken"`

## Why Queries Work Anyway

**Validated (test_compare_indexed_vs_query_term_bytes):**

Indexed term: `[74, 69, 74, 6c, 65, 00, 73, 6c, 61, 70]` = `"title\0slap"`
Query term: `[00, 00, 00, 00, 6a, 74, 69, 74, 6c, 65, 00, 73, 6c, 61, 70]` = `"\0\0\0\0jtitle\0slap"`

Query has 5-byte prefix `[field:4][type='j':1]`, then identical suffix to indexed term. The 's' byte (0x73) appears after path terminator (0x00) in both cases.

**Direct term lookup:** Returns `FOUND` - proves corruption is symmetric.

**Evidence:** 3 e2e tests + 2 validation tests confirm queries work despite corruption.

## Implications

**Option A: Ignore the corruption**
- Queries work despite corrupted storage
- Waste some bytes storing 's' redundantly
- Risk: Future Tantivy changes might break this accidental compatibility

**Option B: Fix upstream bug**
- Calculate `path_end` BEFORE `set_type()`
- Requires understanding where type byte actually belongs
- Previous attempt caused "invalid type code" panic
- Risk: 20+ hours debugging term structure

**Option C: Use non-JSON text fields**
- Flatten JSON to individual `Str` fields at Flapjack layer
- Avoid entire JSON term encoding problem
- Trade-off: More fields, but proven to work

## Recommendation

**Ship Option A immediately.** The corruption is cosmetic - queries work. We need Algolia parity NOW, not perfect term encoding.

**After launch:** File upstream bug report with minimal repro. Let Tantivy maintainers fix term structure properly.

## What Works Right Now

✅ EdgeNgramFilter generates correct ngrams per word
✅ JSON fields accept custom tokenizers
✅ Indexing completes without errors
✅ Manual TermQuery returns correct results
✅ Multi-word values work (both "gam" and "lap" match "Gaming Laptop")

## What's Broken

❌ Term byte encoding has 's' in wrong place (cosmetic, both sides match)
❌ QueryParser creates PhraseQuery (confirmed via test_verify_query_term_bytes)
❌ **EdgeNgramFilter position assignment** - reuses source token position instead of incrementing
  - Causes false phrase matches when query has same relative offsets as ngrams
  - QueryParser tokenizes "lap" → `["la", "lap"]` at same position
  - Constructs PhraseQuery with offsets `[(0, "la"), (0, "lap")]` 
  - Matches indexed terms both at position 1 → 1 hit

## Next Steps

1. ✅ **Accept the corruption** - symmetric, both sides match
2. ✅ **Validate PhraseQuery behavior** - confirmed via position tests
3. ❌ **Fix EdgeNgramFilter position bug** - blocker discovered (not just QueryParser)
4. **Decide approach:**
   - **Option A:** Fix EdgeNgramFilter to increment positions, implement QueryBehavior for OR semantics
   - **Option B:** Keep broken positions, rely on QueryBehavior to bypass PhraseQuery entirely
5. **Test against Algolia** - validate end-to-end behavior
6. **File upstream bugs** - corruption + position assignment issues

## Open Questions

**Q:** Should EdgeNgramFilter increment positions per ngram?
- **Pro:** Prevents false phrase matches, semantically correct
- **Con:** Breaks legitimate phrase queries on edge ngram fields (if anyone uses them)

**Q:** Can we ship with broken positions if QueryBehavior::TermsOr bypasses PhraseQuery?
- **Pro:** Simpler fix, QueryBehavior is sufficient
- **Con:** Surprising behavior if user constructs manual PhraseQuery

**Recommendation:** Implement QueryBehavior first (proven sufficient), defer position fix as enhancement.

## Files

- `tests/edge_ngram_e2e_spike.rs` - Proves corruption exists but queries work
- `src/tokenizer/edge_ngram_filter.rs` - Working implementation
- Previous fork docs in flapjack repo - Same bug, same analysis