=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 06:02:41.877 pm
---------------------
kR6 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
mP2 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
=== END ===
Issue: tokenizers not registered on reloaded index. The test creates index, closes it, reopens - but reopened index has empty tokenizer registry.

**Root cause**: Tokenizers are runtime-only, not persisted in meta.json. When index reopens, must re-register.

**Two approaches:**

1. **Test reflects reality** - user must re-register tokenizers after opening index
2. **Test is wrong** - we're testing schema persistence, not tokenizer registration

Option 1 is correct. The test should register tokenizers on the reloaded index:

```sh nesl
#!nesl [@three-char-SHA-256: kR6]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_kR6'
    let reloaded_index = Index::open_in_dir(index_path).unwrap();
    let reloaded_schema = reloaded_index.schema();
EOT_kR6
new_text = <<'EOT_kR6'
    let reloaded_index = Index::open_in_dir(index_path).unwrap();
    
    let edge_ngram_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
        .build();
    reloaded_index.tokenizers().register("edge_ngram", edge_ngram_tokenizer);
    
    let simple_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .build();
    reloaded_index.tokenizers().register("simple", simple_tokenizer);
    
    let reloaded_schema = reloaded_index.schema();
EOT_kR6
#!end_kR6
```

Also need imports:

```sh nesl
#!nesl [@three-char-SHA-256: mP2]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_mP2'
use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::{Index, doc};
use tempfile::TempDir;
EOT_mP2
new_text = <<'EOT_mP2'
use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::{Index, doc};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tempfile::TempDir;
EOT_mP2
#!end_mP2
```

```bash
cd ~/repos/tantivy
cargo test --test search_tokenizer_persistence
```