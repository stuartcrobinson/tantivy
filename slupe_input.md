=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:32:11.978 pm
---------------------
r4m ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs
t8n ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs
=== END ===
Looking at lines 580-590 and 974-980 in query_parser.rs:

**Line 583:** `get(indexing_options.tokenizer())`  
**Line 977:** `tokenizer: indexing_options.tokenizer().to_owned()`

These need to use `search_tokenizer()` instead of `tokenizer()`.

But first - the test failure shows the actual problem isn't what we thought. The PhraseQuery IS matching (1 hit) when we expected 0. This is the "accidental success" from position bug that the docs mentioned.

**The real issue:** We need QueryParser to use `search_tokenizer()` when tokenizing queries, so query "lap" produces single term `["lap"]` instead of `["la", "lap"]`.

```sh nesl
#!nesl [@three-char-SHA-256: r4m]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs"
old_text = <<'EOT_r4m'
                let mut text_analyzer =
                    self.tokenizer_manager
                        .get(option.tokenizer())
                        .ok_or_else(|| QueryParserError::UnknownTokenizer {
                            field: field_entry.name().to_string(),
                            tokenizer: option.tokenizer().to_string(),
                        })?;
EOT_r4m
new_text = <<'EOT_r4m'
                let mut text_analyzer =
                    self.tokenizer_manager
                        .get(option.search_tokenizer())
                        .ok_or_else(|| QueryParserError::UnknownTokenizer {
                            field: field_entry.name().to_string(),
                            tokenizer: option.search_tokenizer().to_string(),
                        })?;
EOT_r4m
#!end_r4m
```

```sh nesl
#!nesl [@three-char-SHA-256: t8n]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs"
old_text = <<'EOT_t8n'
                let mut text_analyzer = self
                    .tokenizer_manager
                    .get(indexing_options.tokenizer())
                    .ok_or_else(|| QueryParserError::UnknownTokenizer {
                        field: field_name.to_string(),
                        tokenizer: indexing_options.tokenizer().to_string(),
                    })?;
EOT_t8n
new_text = <<'EOT_t8n'
                let mut text_analyzer = self
                    .tokenizer_manager
                    .get(indexing_options.search_tokenizer())
                    .ok_or_else(|| QueryParserError::UnknownTokenizer {
                        field: field_name.to_string(),
                        tokenizer: indexing_options.search_tokenizer().to_string(),
                    })?;
EOT_t8n
#!end_t8n
```

Now find the JSON field path around line 977:

```bash
cd /Users/stuart/repos/tantivy
sed -n '970,985p' src/query/query_parser/query_parser.rs
```