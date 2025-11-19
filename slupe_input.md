=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:34:58.459 pm
---------------------
k3x ✅ replace_text_in_file /Users/stuart/repos/tantivy/src/schema/text_options.rs
=== END ===
Perfect! Now run all tests to ensure no regressions:

```bash
cd /Users/stuart/repos/tantivy
cargo test
```

While that runs, add documentation for the new method:

```sh nesl
#!nesl [@three-char-SHA-256: k3x]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/src/schema/text_options.rs"
old_text = <<'EOT_k3x'
    #[must_use]
    pub fn set_search_tokenizer(mut self, tokenizer_name: &str) -> TextFieldIndexing {
        self.search_tokenizer = Some(TokenizerName::from_name(tokenizer_name));
        self
    }

    pub fn search_tokenizer(&self) -> &str {
        self.search_tokenizer
            .as_ref()
            .unwrap_or(&self.tokenizer)
            .name()
    }
EOT_k3x
new_text = <<'EOT_k3x'
    /// Sets the tokenizer to be used at query time for this field.
    ///
    /// If not set, defaults to the index-time tokenizer. This is useful for fields
    /// that use different tokenization at index vs query time, such as edge ngram
    /// fields where you want prefix matching (edge ngrams at index time) but exact
    /// term matching at query time.
    #[must_use]
    pub fn set_search_tokenizer(mut self, tokenizer_name: &str) -> TextFieldIndexing {
        self.search_tokenizer = Some(TokenizerName::from_name(tokenizer_name));
        self
    }

    /// Returns the tokenizer that will be used for query parsing on this field.
    ///
    /// If a search tokenizer was explicitly set, returns that. Otherwise returns
    /// the index-time tokenizer (ensuring queries and indexing use the same
    /// tokenization by default).
    pub fn search_tokenizer(&self) -> &str {
        self.search_tokenizer
            .as_ref()
            .unwrap_or(&self.tokenizer)
            .name()
    }
EOT_k3x
#!end_k3x
```