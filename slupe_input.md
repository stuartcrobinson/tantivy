=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:24:26.250 pm
---------------------
u9x ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/tokenizer/mod.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: u9x]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/tokenizer/mod.rs"
old_text = <<'EOT_u9x'
pub use self::alphanum_only::AlphaNumOnlyFilter;
pub use self::ascii_folding_filter::AsciiFoldingFilter;
EOT_u9x
new_text = <<'EOT_u9x'
pub use self::alphanum_only::AlphaNumOnlyFilter;
pub use self::ascii_folding_filter::AsciiFoldingFilter;
pub use self::edge_ngram_filter::EdgeNgramFilter;
EOT_u9x
#!end_u9x
```

Now test:

```bash
cd /Users/stuart/repos/tantivy
cargo test --lib tokenizer::edge_ngram_filter -- --nocapture
```