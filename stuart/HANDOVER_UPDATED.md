# Tantivy Fork - Handover Document (Updated 2025-11-19)

**Branch:** nov19
**Status:** Tantivy layer complete, Flapjack integration unvalidated

## What's Implemented

### 1. EdgeNgramFilter (Phase 1)
**File:** `src/tokenizer/edge_ngram_filter.rs`
**Status:** Complete, 6 unit tests passing
**Function:** Generates edge ngrams per word post-tokenization
- "Laptop" → ["la", "lap", "lapt", "lapto", "laptop"]
- Operates on tokenized output (not raw text)
- UTF-8 character boundary safe

### 2. search_tokenizer Field (Phase 3)
**Files:** 
- `src/schema/text_options.rs` - TextFieldIndexing struct
- `src/query/query_parser/query_parser.rs` - 3 call sites modified

**API:**
```rust
TextFieldIndexing::default()
    .set_tokenizer("edge_ngram")           // Index time
    .set_search_tokenizer("simple")        // Query time
    .set_index_option(IndexRecordOption::WithFreqsAndPositions)
```

**Behavior:**
- Defaults to index tokenizer when unset (backward compatible)
- QueryParser calls `field.search_tokenizer()` instead of `field.tokenizer()`
- Enables separate index/query tokenization strategies

### 3. Test Coverage
**Production tests (in `tests/`):**
- `queryparser_edge_ngram_behavior.rs` - Single-word prefix validation
- `search_tokenizer_persistence.rs` - Schema serialization roundtrip
- `search_tokenizer_multi_field.rs` - Field independence
- `search_tokenizer_validation.rs` - Default behavior

**Debug tests (in `tests/debug_spikes/`):**
- `edge_ngram_e2e_spike.rs` - Term format investigation
- `check_token_positions.rs` - Position bug analysis
- `compare_term_bytes.rs` - Term byte structure validation
- `phrase_query_position_debug.rs` - PhraseQuery behavior
- `verify_query_term_bytes.rs` - Query construction debug
- `upstream_json_text_baseline.rs` - Upstream behavior baseline

**Unit tests:**
- `src/tokenizer/edge_ngram_filter.rs` - 6 tests
- `tests/search_tokenizer_unit.rs` - 3 tests (kept as integration)

**Results:** All 949 upstream tests + 10 new tests passing

## Implementation Approach

**Original spec (`TANTIVY_FORK_SPECIFICATION.md`):**
- Proposed `QueryMode` enum (TermsOr, TermsAnd, Phrase)
- Would change query structure from PhraseQuery to BooleanQuery

**Actual implementation:**
- Added `search_tokenizer` field instead
- Bypasses problem at tokenization layer, not query structure layer
- Simpler, less invasive

**Why different:**
- `search_tokenizer` leverages existing tokenizer infrastructure
- No QueryParser query construction logic changes needed
- Cleaner separation: tokenization vs query structure

**Trade-off:**
- Spec approach: explicit control over query semantics
- Implemented approach: implicit via tokenizer selection
- Both solve prefix search, implemented approach less code

## Validated Behavior

**Single-word prefix (test: queryparser_edge_ngram_behavior.rs):**
- Index: "Gaming Laptop" with edge_ngram tokenizer
- Query: "lap" with simple tokenizer
- Result: 1 hit ✅

**Mechanism:**
1. Index time: "Laptop" → ["la", "lap", "lapt", "lapto", "laptop"]
2. Query time: "lap" → ["lap"] (single term, no ngrams)
3. TermQuery for "lap" matches indexed term "lap"

**Multi-word queries:**
- Not tested in Tantivy
- Per Algolia behavior: "gaming laptop" = phrase with last-word prefix
- Flapjack's responsibility to handle via QueryParser configuration

## Known Issues

### 1. Term Format (Cosmetic, Non-Blocking)
**Symptom:** Terms indexed as `"title\0slap"` instead of `"title\0lap"`
**Analysis:** 's' byte is Type::Str indicator, not corruption
**Impact:** None - queries work, both sides construct identical terms
**Evidence:** `tests/debug_spikes/compare_term_bytes.rs`
**Decision:** Ship as-is, cosmetic only

### 2. Position Assignment (Documented, Ignored)
**Symptom:** EdgeNgramFilter assigns same position to all ngrams from one word
**Expected:** Increment position per ngram
**Impact:** PhraseQuery accidentally works due to matching relative offsets
**Why we don't care:** search_tokenizer bypasses PhraseQuery entirely
**Evidence:** `tests/debug_spikes/check_token_positions.rs`

### 3. Multi-Word Query Semantics (Out of Scope)
**Not implemented:** prefixLast vs prefixAll modes
**Reason:** QueryParser configuration, not tokenizer concern
**Algolia behavior:** Only last query word treated as prefix by default
**Flapjack action required:** Configure QueryParser to match Algolia semantics

## Critical Gap: Flapjack Integration

**Unvalidated assumption:** JSON fields work with custom tokenizers via public API

**What we tested:**
```rust
// In Tantivy tests
let text_indexing = TextFieldIndexing::default()
    .set_tokenizer("edge_ngram")
    .set_search_tokenizer("simple");

let json_options = JsonObjectOptions::default()
    .set_indexing_options(text_indexing);
```

**What we didn't test:**
- Does Flapjack configure schemas this way?
- Does Flapjack's document indexing pipeline respect these settings?
- Does Flapjack's query parser use search_tokenizer?

**Risk:** Flapjack may wrap Tantivy differently, exposing different APIs. Our changes might be inaccessible or insufficient.

**Evidence of risk:**
- Original spec proposed QueryMode enum (never built)
- Spec may have been based on Flapjack's actual needs
- Our implementation diverged without validating Flapjack constraints

## Next Steps

### Immediate (Before Declaring Complete)

1. **Write Flapjack integration test**
   - Location: Flapjack repo
   - Test: Index JSON doc, query with prefix, verify hit
   - Validates public API, not just internals
   - If fails: reveals actual Flapjack requirements

2. **If Flapjack test passes:**
   - Update Flapjack Cargo.toml to use fork
   - Ship to production
   - Monitor real-world behavior

3. **If Flapjack test fails:**
   - Debug actual failure mode
   - May need QueryMode approach after all
   - Or different API entirely

### Documentation Updates (After Flapjack Validation)

1. Update `TANTIVY_FORK_SPECIFICATION.md` to reflect actual implementation
2. Remove QueryMode references (never built)
3. Document search_tokenizer approach
4. Add Flapjack integration guide
5. Update CHANGELOG

### Future Enhancements (Post-Launch)

1. **Multi-word query modes:**
   - prefixAll: all words treated as prefixes
   - prefixNone: exact match only
   - Per-field configuration

2. **QueryParser improvements:**
   - Explicit query mode control (resurrect QueryMode enum?)
   - Better phrase query handling with ngrams
   - Performance optimization

3. **Upstream contribution:**
   - File PR to Tantivy with search_tokenizer
   - Discuss term format 's' byte (cosmetic fix)
   - EdgeNgramFilter position increment (correctness)

## Architecture Context

**Tantivy = Lucene equivalent:**
- Low-level indexing engine
- Tokenization, term storage, inverted index
- No opinions on query semantics

**Flapjack = Elasticsearch equivalent:**
- Query parsing, ranking, distribution
- API compatibility layer
- Business logic (Algolia parity)

**Division of responsibility:**
- Tantivy: tokenization strategies (what we built)
- Flapjack: query structure, ranking, multi-word semantics
- Both needed for Algolia parity

## Success Criteria

Fork is production-ready when:
- [x] EdgeNgramFilter implemented
- [x] search_tokenizer field added
- [x] QueryParser uses search_tokenizer
- [x] Single-word prefix search works in Tantivy tests
- [x] All upstream tests pass
- [ ] **Flapjack integration test passes** ← BLOCKER
- [ ] Multi-word query behavior documented
- [ ] Performance validated at scale

## Files Changed

**Core implementation:**
- `src/tokenizer/edge_ngram_filter.rs` (new, 190 lines)
- `src/tokenizer/mod.rs` (modified, +2 lines)
- `src/schema/text_options.rs` (modified, +32 lines)
- `src/query/query_parser/query_parser.rs` (modified, 3 call sites)

**Tests (production):**
- `tests/queryparser_edge_ngram_behavior.rs` (new)
- `tests/search_tokenizer_persistence.rs` (new)
- `tests/search_tokenizer_multi_field.rs` (new)
- `tests/search_tokenizer_validation.rs` (new)
- `tests/search_tokenizer_unit.rs` (new)

**Tests (debug, can delete):**
- `tests/debug_spikes/*` (6 files)

**Documentation:**
- `stuart/STATUS.md`
- `stuart/PHASE3_STATUS.md`
- `stuart/ALGOLIA_QUERY_BEHAVIOR.md`
- `stuart/IMPLEMENTATION_PLAN.md`
- `stuart/TERM_FORMAT_CLARIFICATION.md`

## Open Questions

1. **Does Flapjack need QueryMode enum?**
   - Multi-word queries require explicit OR/AND control
   - search_tokenizer doesn't address query structure
   - May need both approaches

2. **Performance impact?**
   - Edge ngrams increase index size ~81%
   - Query performance with many small terms?
   - Need scale testing (100K+ docs)

3. **Upstream contribution strategy?**
   - Maintain fork indefinitely?
   - Quarterly upstream merges?
   - File PR for search_tokenizer feature?

4. **What if Algolia query semantics are more complex?**
   - prefixLast is default, but configurable
   - Advanced features: typo tolerance, synonyms, ranking
   - Do we need deeper QueryParser changes?

## Bottom Line

**Tantivy layer:** Complete and tested
**Flapjack integration:** Unvalidated assumption
**Recommendation:** Write Flapjack test before updating any other docs
**Risk:** 20 hours invested, but may have built wrong abstraction

We built what we thought Flapjack needs. Now verify Flapjack actually needs it.