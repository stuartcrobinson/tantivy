# Algolia Query Behavior - Empirical Findings

**Date:** 2025-11-19
**Method:** Live API testing
**Conclusion:** Queries are NOT tokenized to ngrams. Only last query word treated as prefix (prefixLast).

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

## Multi-Word Query Behavior (Validated 2025-11-19)

**Test Results:**
- "gaming laptop" → 1 hit (Gaming Laptop) - both words must match, phrase-like
- "gam lap" → 0 hits - first word needs complete match (prefixLast)
- "laptop gaming" → 1 hit - order flexibility via proximity ranking
- "mouse stand" → 0 hits - AND semantics, both must be in same doc

**Algolia Default: prefixLast**
- Only last query word treated as prefix
- Earlier words require complete word matches
- Multi-word = phrase matching with proximity ranking
- This is Flapjack/QueryParser responsibility, not Tantivy tokenizer

**Tantivy Implementation:**
- Tokenizer: EdgeNgram at index, simple at query (correct)
- QueryParser: Creates appropriate query structure (BooleanQuery, PhraseQuery)
- Ranking/proximity: Flapjack layer concern

## Evidence

`stuart/research/algolia_test_1/critical_test_v2.js` - Tests 5-9 validate behavior.