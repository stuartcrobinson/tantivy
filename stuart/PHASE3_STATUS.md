# Phase 3: QueryBehavior Implementation - Status

**Date:** 2025-11-19
**Status:** READY - EdgeNgramFilter correct, need search tokenizer

**Validated:** EdgeNgramFilter works correctly at index time. Algolia uses separate search analyzer.

**Implementation:** Add search_tokenizer field to TextFieldIndexing. Defaults to index tokenizer (no breaking changes).

Proceeding with Phase 3 implementation.

## What's Working

✅ EdgeNgramFilter generates correct ngrams: `["la", "lap", "lapt", "lapto", "laptop"]`
✅ Terms indexed (with cosmetic corruption): `"title\0sla"`, `"title\0slap"`, etc.
✅ Manual TermQuery matches: 1 hit on "lap" query
✅ Multi-word values work: Both "gam" and "lap" match "Gaming Laptop"

## The Problem

QueryParser creates PhraseQuery for multi-token inputs:
```rust
// User query: "lap"
// EdgeNgram tokenizer produces: ["la", "lap"] at SAME position
// QueryParser creates: PhraseQuery with offsets [(0, "la"), (0, "lap")]
// In "Gaming Laptop": Both indexed at position 1
// PhraseQuery matches because relative offsets align
// Result: 1 hit (accidentally works!)
```

**But this breaks with different query patterns:**
```rust
// If QueryParser used offsets [(0, "la"), (1, "lap")]:
// Would require consecutive positions → 0 hits
```

**Real issue:** EdgeNgramFilter assigns same position to all ngrams from one word. PhraseQuery accidentally works because query and index have matching relative positions. Fragile - any change to QueryParser offset logic breaks it.

## The Solution

Add optional search_tokenizer field to TextFieldIndexing:

```rust
pub struct TextFieldIndexing {
    record: IndexRecordOption,
    fieldnorms: bool,
    tokenizer: TokenizerName,
    search_tokenizer: Option<TokenizerName>,  // NEW
}

impl TextFieldIndexing {
    pub fn set_search_tokenizer(mut self, name: &str) -> Self {
        self.search_tokenizer = Some(TokenizerName::from_name(name));
        self
    }
    
    pub fn search_tokenizer(&self) -> &str {
        self.search_tokenizer.as_ref()
            .unwrap_or(&self.tokenizer)
            .name()
    }
}
```

Usage in Flapjack:
```rust
TextFieldIndexing::default()
    .set_tokenizer("edge_ngram")        // Index time
    .set_search_tokenizer("simple")     // Query time
```

**Why this solves the problem:**
1. Current behavior is fragile (relies on position bug)
2. Explicit OR semantics clearer than accidental phrase match
3. Enables proper phrase query support if positions get fixed later
4. Algolia-compatible behavior (treats prefix as OR of ngrams, not phrase)