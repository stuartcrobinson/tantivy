=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 05:59:53.407 pm
---------------------
8mK ✅ write_file /Users/stuart/repos/tantivy/tests/search_tokenizer_unit.rs
pL9 ✅ write_file /Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs
xR4 ✅ write_file /Users/stuart/repos/tantivy/tests/search_tokenizer_multi_field.rs
nQ7 ✅ write_file /Users/stuart/repos/tantivy/tests/search_tokenizer_validation.rs
=== END ===
## Gap analysis

**What's proven:**
- ✅ Single test validates: index with edge_ngram, query with simple, "lap" matches "Laptop"
- ✅ Test passes (1 hit as expected)

**Critical missing tests:**

1. **search_tokenizer field unit tests** - zero coverage on the actual schema field
2. **Default behavior** - what happens when search_tokenizer unset?
3. **Multiple query terms** - "gam lap" behavior undefined
4. **Schema persistence** - does search_tokenizer survive index reload?
5. **Per-field independence** - field A edge_ngram, field B normal, both work?

**Biggest risk:** The one passing test might be accident. Need to prove:
- Setting search_tokenizer actually changes behavior vs not setting it
- Default fallback to index tokenizer works
- Serialization roundtrip preserves the field

Writing tests now:

```sh nesl
#!nesl [@three-char-SHA-256: 8mK]
action = "write_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_unit.rs"
content = <<'EOT_8mK'
use tantivy::schema::{TextFieldIndexing, IndexRecordOption};

#[test]
fn test_search_tokenizer_defaults_to_index_tokenizer() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram");
    
    assert_eq!(indexing.search_tokenizer(), "edge_ngram", 
        "When search_tokenizer not set, should return index tokenizer");
}

#[test]
fn test_search_tokenizer_explicit_override() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_search_tokenizer("simple");
    
    assert_eq!(indexing.search_tokenizer(), "simple",
        "Explicit search_tokenizer should override default");
}

#[test]
fn test_search_tokenizer_chain_builder() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        .set_search_tokenizer("simple");
    
    assert_eq!(indexing.tokenizer(), "edge_ngram");
    assert_eq!(indexing.search_tokenizer(), "simple");
}
EOT_8mK
#!end_8mK
```

```sh nesl
#!nesl [@three-char-SHA-256: pL9]
action = "write_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_persistence.rs"
content = <<'EOT_pL9'
use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::{Index, doc};
use tempfile::TempDir;

#[test]
fn test_search_tokenizer_persists_across_reload() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path();
    
    let json_field = {
        let mut schema_builder = Schema::builder();
        
        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer("edge_ngram")
            .set_search_tokenizer("simple")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        
        let json_options = JsonObjectOptions::default()
            .set_indexing_options(text_indexing);
        
        let field = schema_builder.add_json_field("data", json_options);
        let schema = schema_builder.build();
        
        let index = Index::create_in_dir(index_path, schema.clone()).unwrap();
        let mut writer = index.writer(50_000_000).unwrap();
        writer.add_document(doc!(field => serde_json::json!({"title": "Test"}))).unwrap();
        writer.commit().unwrap();
        
        field
    };
    
    let reloaded_index = Index::open_in_dir(index_path).unwrap();
    let reloaded_schema = reloaded_index.schema();
    let reloaded_field = reloaded_schema.get_field_entry(json_field);
    
    let json_options = reloaded_field.field_type().as_json().unwrap();
    let text_indexing = json_options.get_text_indexing_options().unwrap();
    
    assert_eq!(text_indexing.tokenizer(), "edge_ngram",
        "Index tokenizer should persist");
    assert_eq!(text_indexing.search_tokenizer(), "simple",
        "Search tokenizer should persist across reload");
}

#[test]
fn test_backward_compat_missing_search_tokenizer() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default");
    
    let json_options = JsonObjectOptions::default()
        .set_indexing_options(text_indexing);
    
    schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema);
    let field = index.schema().get_field("data").unwrap();
    let field_entry = index.schema().get_field_entry(field);
    let json_opts = field_entry.field_type().as_json().unwrap();
    let indexing = json_opts.get_text_indexing_options().unwrap();
    
    assert_eq!(indexing.search_tokenizer(), "default",
        "Old schemas without search_tokenizer should default to index tokenizer");
}
EOT_pL9
#!end_pL9
```

```sh nesl
#!nesl [@three-char-SHA-256: xR4]
action = "write_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_multi_field.rs"
content = <<'EOT_xR4'
use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;

#[test]
fn test_multiple_fields_independent_tokenizers() {
    let mut schema_builder = Schema::builder();
    
    let edge_ngram_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_search_tokenizer("simple")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let normal_indexing = TextFieldIndexing::default()
        .set_tokenizer("simple")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let prefix_field = schema_builder.add_json_field("prefix_data", 
        JsonObjectOptions::default()
            .set_stored()
            .set_indexing_options(edge_ngram_indexing));
    
    let exact_field = schema_builder.add_json_field("exact_data",
        JsonObjectOptions::default()
            .set_stored()
            .set_indexing_options(normal_indexing));
    
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    
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
    writer.add_document(doc!(
        prefix_field => serde_json::json!({"title": "Laptop"}),
        exact_field => serde_json::json!({"sku": "LAP123"})
    )).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    
    let parser = QueryParser::for_index(&index, vec![prefix_field, exact_field]);
    
    let prefix_query = parser.parse_query("prefix_data.title:lap").unwrap();
    let prefix_results = searcher.search(&prefix_query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(prefix_results.len(), 1, 
        "Prefix search on edge_ngram field should match");
    
    let exact_prefix_query = parser.parse_query("exact_data.sku:lap").unwrap();
    let exact_prefix_results = searcher.search(&exact_prefix_query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(exact_prefix_results.len(), 0,
        "Prefix search on exact-match field should NOT match");
    
    let exact_full_query = parser.parse_query("exact_data.sku:lap123").unwrap();
    let exact_full_results = searcher.search(&exact_full_query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(exact_full_results.len(), 1,
        "Full match on exact-match field should work");
}
EOT_xR4
#!end_xR4
```

```sh nesl
#!nesl [@three-char-SHA-256: nQ7]
action = "write_file"
path = "/Users/stuart/repos/tantivy/tests/search_tokenizer_validation.rs"
content = <<'EOT_nQ7'
use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;

#[test]
fn test_without_search_tokenizer_uses_index_tokenizer() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let json_options = JsonObjectOptions::default()
        .set_stored()
        .set_indexing_options(text_indexing);
    
    let json_field = schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema.clone());
    
    let tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
        .build();
    index.tokenizers().register("edge_ngram", tokenizer);
    
    let mut writer = index.writer(50_000_000).unwrap();
    writer.add_document(doc!(json_field => serde_json::json!({"title": "Laptop"}))).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let query = query_parser.parse_query("data.title:lap").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    assert_eq!(results.len(), 1,
        "Without search_tokenizer set, query should use edge_ngram (index tokenizer) and create PhraseQuery that accidentally works");
}

#[test]
fn test_search_tokenizer_different_behavior() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_search_tokenizer("simple")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let json_options = JsonObjectOptions::default()
        .set_stored()
        .set_indexing_options(text_indexing);
    
    let json_field = schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    let index = Index::create_in_ram(schema);
    
    let edge_ngram = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
        .build();
    index.tokenizers().register("edge_ngram", edge_ngram);
    
    let simple = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .build();
    index.tokenizers().register("simple", simple);
    
    let mut writer = index.writer(50_000_000).unwrap();
    writer.add_document(doc!(json_field => serde_json::json!({"title": "Gaming Laptop"}))).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let lap_query = query_parser.parse_query("data.title:lap").unwrap();
    let lap_results = searcher.search(&lap_query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(lap_results.len(), 1, "Single term 'lap' should match via direct term lookup");
    
    let gam_query = query_parser.parse_query("data.title:gam").unwrap();
    let gam_results = searcher.search(&gam_query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(gam_results.len(), 1, "Single term 'gam' should match first word prefix");
}
EOT_nQ7
#!end_nQ7
```

Run these:

```bash
cd ~/repos/tantivy
cargo test --test search_tokenizer_unit
cargo test --test search_tokenizer_persistence
cargo test --test search_tokenizer_multi_field
cargo test --test search_tokenizer_validation
```