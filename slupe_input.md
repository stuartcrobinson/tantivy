=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:33:29.581 pm
---------------------
p9w ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: p9w]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/query/query_parser/query_parser.rs"
old_text = <<'EOT_p9w'
    let mut text_analyzer = tokenizer_manager
        .get(text_options.tokenizer())
        .ok_or_else(|| QueryParserError::UnknownTokenizer {
            field: field_name.to_string(),
            tokenizer: text_options.tokenizer().to_string(),
        })?;
EOT_p9w
new_text = <<'EOT_p9w'
    let mut text_analyzer = tokenizer_manager
        .get(text_options.search_tokenizer())
        .ok_or_else(|| QueryParserError::UnknownTokenizer {
            field: field_name.to_string(),
            tokenizer: text_options.search_tokenizer().to_string(),
        })?;
EOT_p9w
#!end_p9w
```

Now test:

```bash
cd /Users/stuart/repos/tantivy
cargo test --test queryparser_edge_ngram_behavior
```

This should now fail differently - query should produce 0 hits because "lap" won't be tokenized to ngrams at query time.