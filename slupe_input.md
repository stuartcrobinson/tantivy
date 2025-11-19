=== SLUPE RESULTS ===
📋 Output copied to clipboard @ 03:47:48.987 pm
---------------------
k8m ✅ write_file /Users/stuart/repos/tantivy/tests/upstream_json_text_baseline.rs
=== END ===
# Upstream JSON Text Validation Spike

Creating baseline validation against clean Tantivy to determine if JSON TEXT fields work at all.

```sh nesl
#!nesl [@three-char-SHA-256: k8m]
action = "write_file"
path = "/Users/stuart/repos/tantivy/tests/upstream_json_text_baseline.rs"
content = <<'EOT_k8m'
use tantivy::schema::{JsonObjectOptions, Schema, INDEXED, STORED, TEXT};
use tantivy::{doc, Index, IndexWriter, TantivyDocument};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;

#[test]
fn test_upstream_json_text_single_word_exact() {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("data", TEXT | STORED);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    let doc = doc!(json_field => serde_json::json!({"title": "Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let query = query_parser.parse_query("data.title:Laptop").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    assert_eq!(results.len(), 1, "Single word exact match should work");
}

#[test]
fn test_upstream_json_text_multi_word_exact() {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("data", TEXT | STORED);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    let doc = doc!(json_field => serde_json::json!({"title": "Gaming Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let query = query_parser.parse_query("data.title:Gaming").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(results.len(), 1, "First word should match");
    
    let query = query_parser.parse_query("data.title:Laptop").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    assert_eq!(results.len(), 1, "Second word should match");
}

#[test]
fn test_upstream_json_text_prefix_fails() {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("data", TEXT | STORED);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    let doc = doc!(json_field => serde_json::json!({"title": "Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let query = query_parser.parse_query("data.title:Lap").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    assert_eq!(results.len(), 0, "Prefix search should NOT work without edge ngrams (expected failure)");
}

#[test]
fn test_upstream_json_text_case_insensitive() {
    let mut schema_builder = Schema::builder();
    let json_field = schema_builder.add_json_field("data", TEXT | STORED);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    let doc = doc!(json_field => serde_json::json!({"title": "Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    
    let query = query_parser.parse_query("data.title:laptop").unwrap();
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    assert_eq!(results.len(), 1, "Case insensitive search should work");
}
EOT_k8m
#!end_k8m
```

Run with:

```bash
cd /Users/stuart/repos/tantivy
cargo test --test upstream_json_text_baseline -- --nocapture
```

## What This Tests

1. **Single word exact**: Does "Laptop" match "Laptop"? (Baseline sanity)
2. **Multi-word exact**: Does "Gaming" match in "Gaming Laptop"? (Tokenization works)
3. **Prefix fails**: Does "Lap" match "Laptop"? (Should fail - validates prefix is the problem)
4. **Case insensitive**: Does "laptop" match "Laptop"? (Lowercasing works)

## Expected Results

If upstream is healthy:
- Tests 1, 2, 4 pass
- Test 3 fails (expected - proves prefix search is missing)

If upstream is broken:
- Test 2 fails → JSON text tokenization doesn't work at all
- Test 1 fails → JSON fields fundamentally broken

## Decision Tree

**All pass except test 3**: Upstream healthy, proceed with EdgeNgram + QueryBehavior fork

**Test 2 fails**: Upstream JSON text search broken, file bug, don't fork

**Test 1 fails**: Critical Tantivy bug, escalate to maintainers

Run this and share output - determines if we fork or file upstream bug.