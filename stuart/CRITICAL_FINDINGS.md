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

**Hypothesis:** Term matching is NOT byte-exact comparison. Either:
1. Query construction also adds 's' incorrectly (both sides wrong, accidentally match)
2. Term lookup normalizes/strips type byte during comparison
3. Serialization layer reconciles the difference

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

**Theory:** `Term::from_field_json_path()` + `append_type_and_str()` constructs:
```
[field][j]["title"][null][s]["lap"]
```

During lookup, path string "title" gets converted to same unordered_id used during indexing. The 's' placement matches on both sides.

**Evidence:** All 3 e2e tests show:
- Corruption visible in raw term bytes
- Queries return correct results (1 hit)

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

❌ Term byte encoding has 's' in wrong place
❌ QueryParser creates PhraseQuery (returns 0 hits) - already known, needs QueryBehavior enum

## Next Steps

1. **Accept the corruption** - it's cosmetic and upstream's problem
2. **Implement QueryBehavior enum** (Phase 3) - this is the real blocker
3. **Test against Algolia** - validate end-to-end behavior
4. **Ship to production** - corruption doesn't affect functionality
5. **File upstream bug** - with test case showing corruption (but working queries)

## Files

- `tests/edge_ngram_e2e_spike.rs` - Proves corruption exists but queries work
- `src/tokenizer/edge_ngram_filter.rs` - Working implementation
- Previous fork docs in flapjack repo - Same bug, same analysis