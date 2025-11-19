=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:47:31.740 pm
---------------------
m8j ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/CRITICAL_FINDINGS.md
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: m8j]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/CRITICAL_FINDINGS.md"
old_text = <<'EOT_m8j'
## Why Queries Work Anyway

**Theory:** `Term::from_field_json_path()` + `append_type_and_str()` constructs:
```
[field][j]["title"][null][s]["lap"]
```

During lookup, path string "title" gets converted to same unordered_id used during indexing. The 's' placement matches on both sides.

**Evidence:** All 3 e2e tests show:
- Corruption visible in raw term bytes
- Queries return correct results (1 hit)
EOT_m8j
new_text = <<'EOT_m8j'
## Why Queries Work Anyway

**Validated (test_compare_indexed_vs_query_term_bytes):**

Indexed term: `[74, 69, 74, 6c, 65, 00, 73, 6c, 61, 70]` = `"title\0slap"`
Query term: `[00, 00, 00, 00, 6a, 74, 69, 74, 6c, 65, 00, 73, 6c, 61, 70]` = `"\0\0\0\0jtitle\0slap"`

Query has 5-byte prefix `[field:4][type='j':1]`, then identical suffix to indexed term. The 's' byte (0x73) appears after path terminator (0x00) in both cases.

**Direct term lookup:** Returns `FOUND` - proves corruption is symmetric.

**Evidence:** 3 e2e tests + 2 validation tests confirm queries work despite corruption.
EOT_m8j
#!end_m8j
```

Docs updated. Phase 3 next?

**Critical question before implementation:** Do we fix EdgeNgramFilter position bug or accept it?

**Option A - Fix positions (4-6h additional):**
- EdgeNgramFilter increments position per ngram
- Semantically correct
- Breaks current accidental PhraseQuery behavior
- Requires QueryBehavior anyway for OR semantics

**Option B - Keep broken positions (0h):**
- Ship QueryBehavior::TermsOr only
- Position bug becomes moot (BooleanQuery ignores positions)
- Simpler, faster to ship
- Manual PhraseQuery on edge ngram fields gives surprising results (edge case)

**Recommendation:** Option B. Position bug only affects PhraseQuery, which shouldn't be used on prefix search fields. QueryBehavior makes positions irrelevant for intended use case.

Proceed with Phase 3 QueryBehavior implementation?