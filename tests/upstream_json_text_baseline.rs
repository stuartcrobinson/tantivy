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