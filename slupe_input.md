=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:43:19.476 pm
---------------------
k8m ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/check_token_positions.rs
p3w ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/check_token_positions.rs
r5n ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/check_token_positions.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: k8m]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/check_token_positions.rs"
old_text = <<'EOT_k8m'
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, Token, WhitespaceTokenizer};
use tantivy::{doc, Index};
EOT_k8m
new_text = <<'EOT_k8m'
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::Index;
EOT_k8m
#!end_k8m
```

```sh nesl
#!nesl [@three-char-SHA-256: p3w]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/check_token_positions.rs"
old_text = <<'EOT_p3w'
    let json_field = schema_builder.add_json_field("data", json_options);
EOT_p3w
new_text = <<'EOT_p3w'
    let _json_field = schema_builder.add_json_field("data", json_options);
EOT_p3w
#!end_p3w
```

```sh nesl
#!nesl [@three-char-SHA-256: r5n]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/check_token_positions.rs"
old_text = <<'EOT_r5n'
    let tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
EOT_r5n
new_text = <<'EOT_r5n'
    let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
EOT_r5n
#!end_r5n
```

```bash
cd /Users/stuart/repos/tantivy
cargo test --test check_token_positions -- --nocapture
```