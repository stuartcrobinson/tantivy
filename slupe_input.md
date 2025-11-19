=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 04:44:37.267 pm
---------------------
t4h ✅ replace_text_in_file /Users/stuart/repos/tantivy/tests/phrase_query_position_debug.rs
=== END ===
```sh nesl
#!nesl [@three-char-SHA-256: t4h]
action = "replace_text_in_file"
path = "/Users/stuart/repos/tantivy/tests/phrase_query_position_debug.rs"
old_text = <<'EOT_t4h'
    let phrase_terms_same = vec![(0, Term::from_field_json_path(json_field, "title", false).with_str("la")), 
                                  (0, Term::from_field_json_path(json_field, "title", false).with_str("lap"))];
EOT_t4h
new_text = <<'EOT_t4h'
    let mut term_la_same = Term::from_field_json_path(json_field, "title", false);
    term_la_same.append_type_and_str("la");
    let mut term_lap_same = Term::from_field_json_path(json_field, "title", false);
    term_lap_same.append_type_and_str("lap");
    let phrase_terms_same = vec![(0, term_la_same), (0, term_lap_same)];
EOT_t4h
#!end_t4h
```

```bash
cd /Users/stuart/repos/tantivy
cargo test --test phrase_query_position_debug -- --nocapture
```