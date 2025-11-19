=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:28:46.550 pm
---------------------
m2n ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/schema/text_options.rs
p8x ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/schema/text_options.rs
t7k ✅ replace_text_in_file /Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: m2n]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/schema/text_options.rs"
old_text = <<'EOT_m2n'
pub const STRING: TextOptions = TextOptions {
    indexing: Some(TextFieldIndexing {
        tokenizer: TokenizerName::from_static(NO_TOKENIZER_NAME),
        fieldnorms: true,
        record: IndexRecordOption::Basic,
    }),
    stored: false,
    fast: FastFieldTextOptions::IsEnabled(false),
    coerce: false,
};
EOT_m2n
new_text = <<'EOT_m2n'
pub const STRING: TextOptions = TextOptions {
    indexing: Some(TextFieldIndexing {
        tokenizer: TokenizerName::from_static(NO_TOKENIZER_NAME),
        fieldnorms: true,
        record: IndexRecordOption::Basic,
        search_tokenizer: None,
    }),
    stored: false,
    fast: FastFieldTextOptions::IsEnabled(false),
    coerce: false,
};
EOT_m2n
#!end_m2n
```

```sh nesl
#!nesl [@three-char-SHA-256: p8x]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/schema/text_options.rs"
old_text = <<'EOT_p8x'
pub const TEXT: TextOptions = TextOptions {
    indexing: Some(TextFieldIndexing {
        tokenizer: TokenizerName::from_static(DEFAULT_TOKENIZER_NAME),
        fieldnorms: true,
        record: IndexRecordOption::WithFreqsAndPositions,
    }),
    stored: false,
    coerce: false,
    fast: FastFieldTextOptions::IsEnabled(false),
};
EOT_p8x
new_text = <<'EOT_p8x'
pub const TEXT: TextOptions = TextOptions {
    indexing: Some(TextFieldIndexing {
        tokenizer: TokenizerName::from_static(DEFAULT_TOKENIZER_NAME),
        fieldnorms: true,
        record: IndexRecordOption::WithFreqsAndPositions,
        search_tokenizer: None,
    }),
    stored: false,
    coerce: false,
    fast: FastFieldTextOptions::IsEnabled(false),
};
EOT_p8x
#!end_p8x
```

```sh nesl
#!nesl [@three-char-SHA-256: t7k]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/stuart/PHASE3_STATUS.md"
old_text = <<'EOT_t7k'
**Why this solves the problem:**
1. Current behavior is fragile (relies on position bug)
2. Explicit OR semantics clearer than accidental phrase match
3. Enables proper phrase query support if positions get fixed later
4. Algolia-compatible behavior (treats prefix as OR of ngrams, not phrase)
EOT_t7k
new_text = <<'EOT_t7k'
**Why this solves the problem:**
- Query "lap" uses simple tokenizer (no ngrams) → single term "lap"
- Index has "lap" from EdgeNgram tokenization of "Laptop"
- Direct term match works correctly
- Algolia-compatible behavior

## Implementation Checklist

- [x] Add search_tokenizer field to TextFieldIndexing struct
- [x] Add set_search_tokenizer() and search_tokenizer() methods
- [x] Fix const initializers (STRING, TEXT)
- [ ] Modify QueryParser to call search_tokenizer() instead of tokenizer()
- [ ] Unit test: search_tokenizer defaults to index tokenizer
- [ ] Integration test: edge_ngram + simple = working prefix search

## Acceptance Criteria

- [ ] Query "lap" returns 1 hit on "Gaming Laptop" with separate search tokenizer
- [ ] Query "gam" returns 1 hit on "Gaming Laptop"
- [ ] Fields without search_tokenizer use index tokenizer (backward compatible)
- [ ] Phrase queries still work on non-edge-ngram fields
- [ ] cargo test passes (no regressions)
EOT_t7k
#!end_t7k
```

Now find QueryParser tokenizer usage:

```bash
cd /Users/stuart/repos/tantivy
grep -n "field_entry.*tokenizer\|indexing_options.*tokenizer" src/query/query_parser/query_parser.rs | head -30
```