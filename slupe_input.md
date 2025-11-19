=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 06:03:40.523 pm
---------------------
vN4 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
=== END ===
Line 26 fails because tokenizers aren't registered on first index before `add_document()`. Need to register before writing:

```sh nesl
#!nesl [@three-char-SHA-256: vN4]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_vN4'
        let index = Index::create_in_dir(index_path, schema.clone()).unwrap();
        let mut writer = index.writer(50_000_000).unwrap();
        writer.add_document(doc!(field => serde_json::json!({"title": "Test"}))).unwrap();
EOT_vN4
new_text = <<'EOT_vN4'
        let index = Index::create_in_dir(index_path, schema.clone()).unwrap();
        
        let edge_ngram_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
            .build();
        index.tokenizers().register("edge_ngram", edge_ngram_tokenizer);
        
        let simple_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .build();
        index.tokenizers().register("simple", simple_tokenizer);
        
        let mut writer = index.writer(50_000_000).unwrap();
        writer.add_document(doc!(field => serde_json::json!({"title": "Test"}))).unwrap();
EOT_vN4
#!end_vN4
```

```bash
cd ~/repos/tantivy
cargo test --test search_tokenizer_persistence
```