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