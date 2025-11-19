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