=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:25:09.716 pm
---------------------
v3y ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/tokenizer/edge_ngram_filter.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: v3y]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/tokenizer/edge_ngram_filter.rs"
old_text = <<'EOT_v3y'
pub struct EdgeNgramFilterWrapper<T> {
    inner: T,
    min_gram: usize,
    max_gram: usize,
    ngrams: Vec<Token>,
}
EOT_v3y
new_text = <<'EOT_v3y'
#[derive(Clone)]
pub struct EdgeNgramFilterWrapper<T> {
    inner: T,
    min_gram: usize,
    max_gram: usize,
    ngrams: Vec<Token>,
}
EOT_v3y
#!end_v3y
```

```bash
cd /Users/stuart/repos/tantivy
cargo test --lib tokenizer::edge_ngram_filter -- --nocapture
```