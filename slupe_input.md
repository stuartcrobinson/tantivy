=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:34:21.081 pm
---------------------
w7k ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs
z5m ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs
a2p ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: w7k]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs"
old_text = <<'EOT_w7k'
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
EOT_w7k
new_text = <<'EOT_w7k'
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_search_tokenizer("simple")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
EOT_w7k
#!end_w7k
```

```sh nesl
#!nesl [@three-char-SHA-256: z5m]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs"
old_text = <<'EOT_z5m'
    index.tokenizers().register("edge_ngram", tokenizer);
EOT_z5m
new_text = <<'EOT_z5m'
    index.tokenizers().register("edge_ngram", tokenizer);
    
    let simple_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("simple", simple_tokenizer);
EOT_z5m
#!end_z5m
```

```sh nesl
#!nesl [@three-char-SHA-256: a2p]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/queryparser_edge_ngram_behavior.rs"
old_text = <<'EOT_a2p'
    println!("\n=== EXPECTED BEHAVIOR ===");
    println!("QueryParser tokenizes 'lap' → ['la', 'lap']");
    println!("Creates PhraseQuery requiring consecutive terms");
    println!("In 'Gaming Laptop': 'la' and 'lap' NOT consecutive");
    println!("Expected hits: 0");
    println!("Actual hits: {}", results.len());
    
    // This SHOULD fail if QueryParser creates PhraseQuery
    // If it passes, we have bigger questions about how matching works
    assert_eq!(results.len(), 0, "PhraseQuery should fail on non-consecutive edge ngrams");
EOT_a2p
new_text = <<'EOT_a2p'
    println!("\n=== EXPECTED BEHAVIOR ===");
    println!("Index: EdgeNgram tokenizer generates ['la', 'lap', 'lapt', 'lapto', 'laptop']");
    println!("Query: Simple tokenizer generates ['lap'] (single term)");
    println!("TermQuery for 'lap' matches indexed term 'lap'");
    println!("Expected hits: 1");
    println!("Actual hits: {}", results.len());
    
    assert_eq!(results.len(), 1, "Query 'lap' should match 'Laptop' via prefix");
EOT_a2p
#!end_a2p
```

Now test:

```bash
cd /Users/stuart/repos/tantivy
cargo test --test queryparser_edge_ngram_behavior
```