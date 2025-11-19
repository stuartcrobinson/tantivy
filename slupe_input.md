=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:03:21.384 pm
---------------------
x8k ✅ write_file /Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md
p4v ✅ write_file /Users/stuart/repos/tantivy/stuart/SESSION_2025_11_19_FRESH_START.md
w3n ✅ write_file /Users/stuart/repos/tantivy/stuart/ALGOLIA_VALIDATION.md
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: x8k]
action = "write_file"
path = "/Users/stuart/repos/tantivy/stuart/IMPLEMENTATION_PLAN.md"
content = <<'EOT_x8k'
# Tantivy Fork Implementation Plan - EdgeNgramFilter + QueryBehavior

**Date:** 2025-11-19
**Goal:** Enable Algolia-compatible prefix search on JSON fields
**Validation:** Algolia confirmed per-word prefix matching ("lap" matches "Gaming Laptop")

## Evidence

**Upstream baseline tests** (`tests/upstream_json_text_baseline.rs`):
- ✅ JSON TEXT fields work for exact matching
- ✅ Multi-word tokenization works
- ✅ Case insensitive search works
- ✅ Prefix search correctly fails (no edge ngrams)

**Algolia behavior** (`stuart/research/algolia_test_1/critical_test_v2.js`):
- ✅ Query "lap" matches "Gaming Laptop" (2nd word)
- ✅ Query "gam" matches "Gaming Mouse" (1st word)
- ✅ Query "mou" matches "Gaming Mouse" (prefix)
- **Conclusion:** Algolia does per-word prefix, NOT full-text ngrams

**Implication:** Must implement EdgeNgramFilter (TokenFilter operating post-tokenization), not use NgramTokenizer (operates on full text).

---

## Phase 1: EdgeNgramFilter Implementation (6h)

**Objective:** Build TokenFilter that generates edge ngrams per-word

**Reference:** `SplitCompoundWords` tokenizer (existing pattern for 1-to-many token emission)

### Tasks

- [ ] Create `src/tokenizer/edge_ngram_filter.rs`
  - [ ] Implement `TokenFilter` trait
  - [ ] Generate ngrams from min_gram to max_gram per token
  - [ ] Preserve token positions (prevent false phrase matches)
  - [ ] Handle UTF-8 correctly (character boundaries, not bytes)

- [ ] Register in `src/tokenizer/mod.rs`
  - [ ] Add `pub mod edge_ngram_filter;`
  - [ ] Export `pub use edge_ngram_filter::EdgeNgramFilter;`

- [ ] Unit tests in `edge_ngram_filter.rs`
  - [ ] Test: "gaming" → ["ga", "gam", "gami", "gamin", "gaming"]
  - [ ] Test: min_gram=2, max_gram=10 boundaries
  - [ ] Test: UTF-8 "café" → ["ca", "caf", "café"]
  - [ ] Test: Position preservation across tokens

**Sign-off criteria:**
- `cargo test --lib tokenizer::edge_ngram_filter` passes
- No compilation errors
- Filter operates per-token (not full-text)

---

## Phase 2: Integration Tests (3h)

**Objective:** Validate EdgeNgramFilter works with JSON fields and isolate QueryParser bug

### Tasks

- [ ] Create `tests/edge_ngram_json_integration.rs`

**Test 1: Indexing produces correct terms**
```rust
// Index: {"title": "Laptop"}
// Verify inverted index contains: "la", "lap", "lapt", "lapto", "laptop"
```

**Test 2: Manual TermQuery works**
```rust
// Manual TermQuery for "lap" should match "Gaming Laptop"
// Proves terms are correct, query construction is isolated
```

**Test 3: QueryParser creates PhraseQuery (expected failure)**
```rust
// QueryParser.parse_query("data.title:lap") should return 0 hits
// Documents the bug we're about to fix
```

**Test 4: Multi-word values**
```rust
// Index: {"title": "Gaming Laptop"}
// Manual TermQuery("lap") should match
// Manual TermQuery("gam") should match
```

**Sign-off criteria:**
- Tests 1, 2, 4 pass (indexing works)
- Test 3 fails as expected (QueryParser bug confirmed)
- All tests compile and run in <100ms

---

## Phase 3: QueryBehavior Enum (8h)

**Objective:** Add schema-level control of QueryParser multi-token behavior

### Tasks

- [ ] Add enum to `src/schema/text_options.rs`
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum QueryBehavior {
    Phrase,    // Default: consecutive term requirement
    TermsOr,   // Edge ngrams: OR'd term queries
}

impl Default for QueryBehavior {
    fn default() -> Self { Self::Phrase }
}
```

- [ ] Extend `TextFieldIndexing`
```rust
impl TextFieldIndexing {
    pub fn set_query_behavior(mut self, behavior: QueryBehavior) -> Self {
        self.query_behavior = behavior;
        self
    }
    
    pub fn query_behavior(&self) -> QueryBehavior {
        self.query_behavior.clone()
    }
}
```

- [ ] Modify `src/query/query_parser/query_parser.rs`
  - [ ] Find multi-token handling (~line 960 for text, ~1040 for JSON)
  - [ ] Check field's `query_behavior` before creating PhraseQuery
  - [ ] If `TermsOr`: create `BooleanQuery` with `Occur::Should` per term
  - [ ] If `Phrase`: preserve existing behavior

- [ ] Schema serialization
  - [ ] Add `#[serde(skip_serializing_if = "is_default")]` to avoid meta.json bloat
  - [ ] Verify roundtrip: serialize → deserialize preserves value

- [ ] Update integration tests
  - [ ] Modify Test 3 to use `QueryBehavior::TermsOr`
  - [ ] Verify QueryParser now creates BooleanQuery
  - [ ] Verify query returns 1 hit

**Sign-off criteria:**
- All tests pass (including previously failing Test 3)
- `cargo test` runs without regressions (943+ tests passing)
- QueryParser respects field's query_behavior setting
- Phrase mode still works for non-ngram fields

---

## Phase 4: Documentation (2h)

- [ ] Create `stuart/COMPLETION_SUMMARY.md`
  - [ ] What was built (EdgeNgramFilter, QueryBehavior)
  - [ ] Why (Algolia per-word prefix parity)
  - [ ] Test results (evidence of correctness)
  - [ ] Integration instructions for Flapjack

- [ ] Update `stuart/ALGOLIA_VALIDATION.md`
  - [ ] Document critical_test_v2.js results
  - [ ] Explain per-word vs full-text ngram decision

- [ ] Code comments
  - [ ] Document EdgeNgramFilter algorithm
  - [ ] Explain QueryBehavior use cases
  - [ ] Reference Algolia behavior in comments

**Sign-off criteria:**
- Flapjack team can integrate without asking questions
- Future maintainers understand design rationale
- No ambiguity about when to use TermsOr vs Phrase

---

## Total Estimate: 19 hours

| Phase | Hours | Cumulative |
|-------|-------|------------|
| EdgeNgramFilter | 6 | 6 |
| Integration tests | 3 | 9 |
| QueryBehavior | 8 | 17 |
| Documentation | 2 | 19 |

---

## Risk Mitigation

**Risk:** EdgeNgramFilter position handling breaks phrase queries on non-ngram fields
**Mitigation:** Test existing phrase query tests still pass. Position offsets must increment correctly.

**Risk:** QueryBehavior enum breaks backward compatibility
**Mitigation:** Default to `Phrase`. Existing indices serialize without `query_behavior` field (skipped if default).

**Risk:** UTF-8 boundary bugs in edge ngram generation
**Mitigation:** Comprehensive unit tests with emoji, accents, multi-byte chars.

**Risk:** Query performance degrades with many small terms
**Mitigation:** Acceptable for Algolia parity. Edge ngrams are inherently storage/query heavy.

---

## Success Criteria

Fork complete when:
1. EdgeNgramFilter compiles and passes unit tests
2. JSON fields can use edge_ngram tokenizer via schema API
3. Query "lap" matches document with "Gaming Laptop"
4. Query "gam" matches document with "Gaming Mouse"
5. All upstream tests still pass (no regressions)
6. QueryParser respects QueryBehavior setting
7. Documentation explains integration path for Flapjack

---

## Next Steps

1. Implement Phase 1 (EdgeNgramFilter)
2. Run unit tests, validate correctness
3. Proceed to Phase 2 only if Phase 1 tests pass
4. Do not skip to Phase 3 without Phase 2 validation
5. Update this document with actual hours spent per phase
EOT_x8k
#!end_x8k
```

```sh nesl
#!nesl [@three-char-SHA-256: p4v]
action = "write_file"
path = "/Users/stuart/repos/tantivy/stuart/SESSION_2025_11_19_FRESH_START.md"
content = <<'EOT_p4v'
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
EOT_p4v
#!end_p4v
```

```sh nesl
#!nesl [@three-char-SHA-256: w3n]
action = "write_file"
path = "/Users/stuart/repos/tantivy/stuart/ALGOLIA_VALIDATION.md"
content = <<'EOT_w3n'
# Algolia Prefix Search Behavior - Empirical Validation

**Date:** 2025-11-19
**Method:** Live API testing against Algolia production
**Conclusion:** Per-word prefix matching confirmed

## Test Setup

**Index:** `prefix_behavior_test`
**Documents:**
1. `{"objectID": "1", "title": "Gaming Laptop"}`
2. `{"objectID": "2", "title": "Laptop Stand"}`
3. `{"objectID": "3", "title": "Gaming Mouse"}`

**Code:** `stuart/research/algolia_test_1/critical_test_v2.js`

## Results

### Test 1: Mid-Word Prefix
**Query:** `"lap"`
**Expected:** Matches "Gaming Laptop" (2nd word) and "Laptop Stand" (1st word)
**Result:** ✅ 2 hits - "Laptop Stand", "Gaming Laptop"

### Test 2: First-Word Prefix
**Query:** `"gam"`
**Expected:** Matches "Gaming Mouse" and "Gaming Laptop" (1st word)
**Result:** ✅ 2 hits - "Gaming Mouse", "Gaming Laptop"

### Test 3: Prefix on Single Word
**Query:** `"mou"`
**Expected:** Matches "Gaming Mouse" (2nd word)
**Result:** ✅ 1 hit - "Gaming Mouse"

## Key Finding

Algolia indexes multi-word values **per-word**, not as full text.

**Evidence:**
- "lap" matches 2nd word in "Gaming Laptop"
- If full-text ngrams, would need to search "Gaming L", "Gaming La", "Gaming Lap"
- Query "lap" alone wouldn't match (no standalone "lap" in full-text ngrams)

## Implications for Flapjack

**Must implement:** EdgeNgramFilter (TokenFilter operating post-tokenization)
**Cannot use:** NgramTokenizer (Tokenizer operating on full text)

**Why:**
```
NgramTokenizer("Gaming Laptop"):
  → ["Ga", "Gam", ..., "Gaming L", "Gaming La", "Gaming Lap"]
  Query "lap" → searches for standalone "lap" → NOT FOUND

EdgeNgramFilter(["Gaming", "Laptop"]):
  → ["ga", "gam", ..., "gaming", "la", "lap", ..., "laptop"]
  Query "lap" → searches for "lap" → FOUND (from "Laptop" ngrams)
```

## Reproduction

```bash
cd stuart/research/algolia_test_1
npm install
node critical_test_v2.js
```

**Expected output:**
```
Query "lap": 2 hits
Matched documents:
  - Laptop Stand
  - Gaming Laptop
✅ CONFIRMED: Algolia does per-word prefix matching
```

## References

- Algolia docs: https://www.algolia.com/doc/guides/managing-results/optimize-search-results/override-search-engine-defaults/
- API used: `algoliasearch` v5.0.0 (Node.js client)
- Test credentials: `.env` in same directory (gitignored)
EOT_w3n
#!end_w3n
```

Both implementation plan and session context saved. Ready to start Phase 1 when you return.