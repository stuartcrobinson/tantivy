# Implementation vs Specification Analysis

**Date:** 2025-11-19
**Question:** Why did we build search_tokenizer instead of QueryMode enum?

## Original Specification

**File:** `docs/TANTIVY_FORK_SPECIFICATION.md`
**Proposed:** QueryMode enum

```rust
pub enum QueryMode {
    Phrase,      // PhraseQuery (default)
    TermsOr,     // BooleanQuery with OR
    TermsAnd,    // BooleanQuery with AND
}

impl TextFieldIndexing {
    pub fn set_query_mode(mut self, mode: QueryMode) -> Self;
}
```

**Usage:**
```rust
TextFieldIndexing::default()
    .set_tokenizer("edge_ngram")
    .set_query_mode(QueryMode::TermsOr)
```

**Intent:** Change QueryParser query construction logic for edge ngram fields

## What We Built Instead

**Implementation:** search_tokenizer field

```rust
impl TextFieldIndexing {
    pub fn set_search_tokenizer(mut self, name: &str) -> Self;
    pub fn search_tokenizer(&self) -> &str;
}
```

**Usage:**
```rust
TextFieldIndexing::default()
    .set_tokenizer("edge_ngram")
    .set_search_tokenizer("simple")
```

**Effect:** Separate tokenization at query time vs index time

## Why Different?

**QueryMode approach:**
- Changes query structure (PhraseQuery → BooleanQuery)
- Requires modifying QueryParser query construction logic
- More invasive code changes
- Explicit control over query semantics

**search_tokenizer approach:**
- Changes query tokenization only
- Leverages existing tokenizer infrastructure
- Less code, simpler changes
- Implicit query semantics via tokenizer choice

**Decision rationale (inferred):**
- Discovered during Phase 3 that PhraseQuery accidentally works
- Realized problem was tokenization, not query structure
- Simpler solution: different tokenizer at query time
- Bypasses entire PhraseQuery issue

## Trade-offs

### QueryMode Advantages
1. Explicit control over AND/OR/phrase semantics
2. Works regardless of tokenizer
3. Clear API for multi-word query behavior
4. Matches Algolia's queryType parameter

### search_tokenizer Advantages
1. Less code to maintain
2. Leverages existing tokenizer infrastructure
3. No QueryParser query construction changes
4. Simpler mental model

### QueryMode Disadvantages
1. More invasive changes to QueryParser
2. Adds query structure complexity
3. Potential interactions with other query features

### search_tokenizer Disadvantages
1. Implicit behavior (tokenizer controls query semantics)
2. Doesn't address multi-word query modes directly
3. May still need QueryMode for advanced features
4. Less explicit API

## Did We Build the Right Thing?

**For single-word prefix search:** Yes
- "lap" matches "Laptop" ✅
- Simpler implementation ✅
- All tests pass ✅

**For multi-word queries:** Unknown
- Spec called out prefixLast vs prefixAll
- search_tokenizer doesn't address this
- Flapjack may still need QueryMode

**For Flapjack integration:** Unvalidated
- Spec was written for Flapjack's needs
- We diverged without testing Flapjack
- May have solved wrong problem

## Possible Outcomes

### Scenario 1: search_tokenizer Sufficient
- Flapjack test passes
- Multi-word handled at Flapjack layer
- QueryMode unnecessary
- Ship as-is

### Scenario 2: Need Both
- search_tokenizer works for single-word
- QueryMode still needed for multi-word semantics
- Implement QueryMode as Phase 4
- More work, but correct

### Scenario 3: Wrong Abstraction
- Flapjack needs different API entirely
- search_tokenizer doesn't help
- Revert, implement spec as written
- 20 hours wasted

## Recommendation

**Before any more doc updates:**
1. Write Flapjack integration test
2. Test multi-word queries specifically
3. Validate actual Flapjack requirements

**If search_tokenizer insufficient:**
- Implement QueryMode as originally spec'd
- Keep search_tokenizer (still useful)
- Document why both needed

**If search_tokenizer sufficient:**
- Update spec to reflect reality
- Document why simpler approach works
- Close QueryMode as unnecessary

## Key Insight

We optimized for Tantivy layer simplicity without validating Flapjack layer requirements. Classic systems integration risk.