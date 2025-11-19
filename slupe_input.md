=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:13:47.022 pm
---------------------
r7k ✅ write_file /Users/stuart/repos/tantivy/stuart/ALGOLIA_QUERY_BEHAVIOR.md
m3p ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/STATUS.md
t9w ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: r7k]
action = "write_file"
path = "/Users/stuart/repos/tantivy/stuart/ALGOLIA_QUERY_BEHAVIOR.md"
content = <<'EOT_r7k'
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
EOT_r7k
#!end_r7k
```

```sh nesl
#!nesl [@three-char-SHA-256: m3p]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/STATUS.md"
old_text = <<'EOT_m3p'
**Date:** 2025-11-19
**Branch:** nov19
**Status:** Phase 1-2 Complete ✅ → Ready for Phase 3 (QueryBehavior)

**Current task:** Add QueryBehavior enum to schema layer for OR semantics on edge ngram queries.

**Term format:** Clarified - 's' byte is correct type indicator, not corruption. See `TERM_FORMAT_CLARIFICATION.md`.

**Test status:** 949/949 core tests passing, 3/3 custom edge ngram tests passing
EOT_m3p
new_text = <<'EOT_m3p'
**Date:** 2025-11-19
**Branch:** nov19
**Status:** Phase 1-2 INCOMPLETE - EdgeNgramFilter wrong

**Current task:** Fix EdgeNgramFilter to apply ONLY at index time, not query time.

**Blocker:** Algolia doesn't tokenize queries to ngrams. Query "lap" searches as single term, not `["la", "lap"]`. See `ALGOLIA_QUERY_BEHAVIOR.md`.

**Fix needed:** Separate index/search analyzers in schema. Tantivy supports `set_search_analyzer()` - need to implement.

**Term format:** Clarified - 's' byte is correct type indicator. See `TERM_FORMAT_CLARIFICATION.md`.
EOT_m3p
#!end_m3p
```

```sh nesl
#!nesl [@three-char-SHA-256: t9w]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md"
old_text = <<'EOT_t9w'
**Date:** 2025-11-19
**Status:** READY - Implementing QueryBehavior
**Prerequisite:** EdgeNgramFilter complete ✅, term format understood ✅

**Cleared blocker:** 's' byte is not corruption - it's the correct Type::Str indicator. See `TERM_FORMAT_CLARIFICATION.md`.
EOT_t9w
new_text = <<'EOT_t9w'
**Date:** 2025-11-19
**Status:** BLOCKED - EdgeNgramFilter implementation wrong
**New blocker:** EdgeNgramFilter must NOT run on queries (only index time)

**Discovery:** Algolia query "lap" searches as single term, not tokenized to `["la", "lap"]`. See `ALGOLIA_QUERY_BEHAVIOR.md`.

**Required fix:** Configure separate search analyzer (no EdgeNgramFilter) via schema.

QueryBehavior implementation paused until analyzer separation fixed.
EOT_t9w
#!end_t9w
```

Now check if Tantivy supports separate search analyzers:

```bash
grep -n "search_analyzer\|set_search" src/schema/text_options.rs
```