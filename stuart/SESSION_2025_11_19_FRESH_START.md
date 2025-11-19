# Session 2025-11-19: Fresh Fork Start

## Context

Previous fork attempt (`custom/tokenizer` branch) succeeded but was abandoned due to "weird errors." Branch deleted. Starting over with TDD approach.

**Key lessons from previous attempt:**
- EdgeNgramFilter implementation worked (150 lines)
- QueryBehavior enum worked
- 943/943 tests passing
- "Mysterious success" in queries may have been actual success
- Term "corruption" (`title\0sla`) may have been correct serialization format
- Lack of incremental tests made debugging impossible

## What We Know Now

### Upstream Validation (2025-11-19)

**Test:** `tests/upstream_json_text_baseline.rs` - 4 tests, all pass
- JSON TEXT fields work correctly for exact matching
- Multi-word tokenization works ("Gaming Laptop" → ["gaming", "laptop"])
- Case insensitive search works
- Prefix search correctly fails without edge ngrams

**Conclusion:** Upstream Tantivy is healthy. Fork is justified.

### Algolia Behavior Validation (2025-11-19)

**Test:** `stuart/research/algolia_test_1/critical_test_v2.js`

**Results:**
- Query "lap" → 2 hits: "Laptop Stand", "Gaming Laptop"
- Query "gam" → 2 hits: "Gaming Mouse", "Gaming Laptop"  
- Query "mou" → 1 hit: "Gaming Mouse"

**Critical finding:** Algolia does **per-word** prefix matching, not full-text ngrams.

**Example:**
- Document: "Gaming Laptop"
- Indexed as: ["gaming": ["ga", "gam", "gami", "gamin", "gaming"], "laptop": ["la", "lap", ...]]
- Query "lap" finds "laptop" ngrams (2nd word)

**Implication:** Must use EdgeNgramFilter (operates post-tokenization per word), not NgramTokenizer (operates on full text "Gaming Laptop" → "Gaming L", "Gaming La").

## Technical Decisions

### Why EdgeNgramFilter, Not NgramTokenizer?

**NgramTokenizer behavior:**
```
Input: "Gaming Laptop"
Output: ["Ga", "Gam", "Gami", "Gamin", "Gaming", "Gaming ", "Gaming L", "Gaming La", "Gaming Lap", ...]
```
Query "lap" searches for standalone "lap" token → NOT found in above list.

**EdgeNgramFilter behavior:**
```
Input after WhitespaceTokenizer: ["Gaming", "Laptop"]
Output: ["ga", "gam", "gami", "gamin", "gaming", "la", "lap", "lapt", "lapto", "laptop"]
```
Query "lap" searches for "lap" token → found in ngrams of "Laptop" word.

### Why QueryBehavior Enum?

**Current behavior (without fork):**
- QueryParser tokenizes query "lap" → ["la", "lap"]
- Creates PhraseQuery requiring consecutive terms
- In "Gaming Laptop" indexed as [..., "gaming", "la", "lap", "lapt", "lapto", "laptop", ...]
- Terms "la" and "lap" exist but NOT consecutively (separated by "lapt", "lapto", etc)
- Result: 0 hits

**Required behavior:**
- QueryParser checks field's `query_behavior` setting
- If `TermsOr`: Creates BooleanQuery with `Occur::Should` for each term
- Query becomes: `(Should: TermQuery("la")) OR (Should: TermQuery("lap"))`
- Both terms exist → match

**Alternative considered:** Hardcode detection (if tokenizer name contains "ngram", use OR logic)
**Rejected:** Less explicit, couples tokenizer naming to query behavior

## Implementation Strategy

### Phase 1: Build EdgeNgramFilter (6h)
- Copy pattern from `SplitCompoundWords` (1-to-many token emission)
- Unit tests in same file (fast, isolated)
- No integration until unit tests pass

### Phase 2: Integration Tests (3h)
- Validate indexing produces correct terms
- Prove manual TermQuery works (isolates bug to QueryParser)
- Document QueryParser bug with failing test

### Phase 3: QueryBehavior (8h)
- Minimal enum in text_options.rs
- Modify query_parser.rs at token-handling points
- Make Phase 2's failing test pass

### Phase 4: Documentation (2h)
- Completion summary for Flapjack integration
- Preserve lessons learned (this document)

## What's Different This Time

| Previous Attempt | This Attempt |
|------------------|--------------|
| Implemented EdgeNgramFilter first | Implement EdgeNgramFilter first (same) |
| No unit tests during development | Unit tests before integration |
| Integration tests mixed with debug spikes | Clean separation: tests/ vs research/ |
| Modified term structure experimentally | Don't touch term structure |
| Debugged "corruption" in production code | Validate indexing works before querying |
| No incremental validation | Sign-off criteria per phase |

## Current Status

- [x] Upstream validated (healthy)
- [x] Algolia behavior confirmed (per-word prefix)
- [x] Implementation plan written
- [ ] Phase 1 not started
- [ ] Phase 2 not started
- [ ] Phase 3 not started
- [ ] Phase 4 not started

## Files Created

- `tests/upstream_json_text_baseline.rs` - Baseline validation (keep)
- `stuart/research/algolia_test_1/critical_test_v2.js` - Algolia behavior proof (keep)
- `stuart/IMPLEMENTATION_PLAN.md` - Detailed phase checklist (this is the working doc)
- `stuart/SESSION_2025_11_19_FRESH_START.md` - Context for next session (this file)

## Next LLM Session: Start Here

1. Read `stuart/IMPLEMENTATION_PLAN.md` for task list
2. Read `stuart/SESSION_2025_11_19_FRESH_START.md` for context
3. Begin Phase 1: Implement EdgeNgramFilter
4. Do NOT proceed to Phase 2 until Phase 1 unit tests pass
5. Update IMPLEMENTATION_PLAN.md with actual hours spent

## Open Questions

None. Path is clear: implement EdgeNgramFilter following SplitCompoundWords pattern, validate with unit tests, proceed incrementally.

## Key Constraints

- Breaking changes acceptable (no customers)
- Must match Algolia behavior exactly (don't "improve")
- Test-driven: no integration code without unit test validation
- Fast tests: all tests must run in <100ms (no slow integration)
- Clear sign-off: each phase has objective pass/fail criteria

## References

- Algolia validation: `stuart/research/algolia_test_1/`
- Upstream baseline: `tests/upstream_json_text_baseline.rs`
- Previous attempt docs: `/Users/stuart/repos/flapjack202511/deep_dives/phase_2/0_dynamic_schema/from_first_tantivy_fork_failure/stuart/`
- Pattern to follow: `src/tokenizer/split_compound_words.rs` (SplitCompoundWords)
- Query parser location: `src/query/query_parser/query_parser.rs` ~line 960 (text), ~1040 (JSON)