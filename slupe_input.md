=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 06:02:13.388 pm
---------------------
xT9 ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: xT9]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
old_text = <<'EOT_xT9'
    let field_entry = index.schema().get_field_entry(field);
    let json_opts = match field_entry.field_type() {
EOT_xT9
new_text = <<'EOT_xT9'
    let schema = index.schema();
    let field_entry = schema.get_field_entry(field);
    let json_opts = match field_entry.field_type() {
EOT_xT9
#!end_xT9
```

```bash
cd ~/repos/tantivy
cargo test --test search_tokenizer_persistence
```