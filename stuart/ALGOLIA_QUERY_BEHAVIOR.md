# Algolia Query Behavior - Empirical Findings

**Date:** 2025-11-19
**Method:** Live API testing
**Conclusion:** Queries are NOT tokenized to ngrams

## Test Results

**Document:** `{"title": "La"}` (2 chars, generates only "la" ngram at index time)

**Query:** `"lap"` (3 chars)

**Result:** 0 hits

**Interpretation:** Query "lap" does NOT get tokenized to `["la", "lap"]`. It searches as single term `"lap"`.

## Implications for Tantivy Fork

**Index time:** EdgeNgramFilter ✅
- "Laptop" → `["la", "lap", "lapt", "lapto", "laptop"]`

**Query time:** NO EdgeNgramFilter ❌
- "lap" → `["lap"]` (single term)
- NOT `["la", "lap"]`

## Implementation Requirements

1. **Separate analyzers:**
   - Index analyzer: WhitespaceTokenizer + LowerCase + EdgeNgramFilter
   - Search analyzer: WhitespaceTokenizer + LowerCase (no EdgeNgramFilter)

2. **Schema configuration:**
   ```rust
   TextFieldIndexing::default()
       .set_tokenizer("edge_ngram")        // Index time
       .set_search_analyzer("simple")       // Query time (no ngrams)
   ```

3. **QueryBehavior still needed:**
   - Multi-word query "gaming laptop" should be AND, not PhraseQuery
   - Unrelated to ngram tokenization

## Current Fork Status

**Broken:** EdgeNgramFilter applied at both index and query time.

**Fix required:** Configure separate search analyzer in schema.

## Evidence

`stuart/research/algolia_test_1/critical_test_v2.js` - Test 5 proves query not tokenized.