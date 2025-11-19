=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 06:01:18.181 pm
---------------------
mK3 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
pL8 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
=== END ===
Need to pattern match, not call `.as_json()`. Fixing:

```sh nesl
#!nesl [@three-char-SHA-256: mK3]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_mK3'
    let json_options = reloaded_field.field_type().as_json().unwrap();
    let text_indexing = json_options.get_text_indexing_options().unwrap();
EOT_mK3
new_text = <<'EOT_mK3'
    let json_options = match reloaded_field.field_type() {
        tantivy::schema::FieldType::JsonObject(opts) => opts,
        _ => panic!("Expected JSON field"),
    };
    let text_indexing = json_options.get_text_indexing_options().unwrap();
EOT_mK3
#!end_mK3
```

```sh nesl
#!nesl [@three-char-SHA-256: pL8]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_pL8'
    let json_opts = field_entry.field_type().as_json().unwrap();
    let indexing = json_opts.get_text_indexing_options().unwrap();
EOT_pL8
new_text = <<'EOT_pL8'
    let json_opts = match field_entry.field_type() {
        tantivy::schema::FieldType::JsonObject(opts) => opts,
        _ => panic!("Expected JSON field"),
    };
    let indexing = json_opts.get_text_indexing_options().unwrap();
EOT_pL8
#!end_pL8
```

```bash
cd ~/repos/tantivy
cargo test --test search_tokenizer_persistence
```