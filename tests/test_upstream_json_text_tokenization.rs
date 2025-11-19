use tantivy::schema::{JsonObjectOptions, Schema, INDEXED, STORED, TEXT};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};

#[test]
fn test_upstream_json_text_with_edge_ngrams() {
    let mut schema_builder = Schema::builder();
    
    let json_options = JsonObjectOptions::default()
        .set_indexed()
        .set_stored();
    
    schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    
    let edge_ngram = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(NgramTokenizer::prefix_only(2, 10).unwrap())
        .build();
    
    index.tokenizers().register("edge_ngram", edge_ngram);
    
    let data_field = schema.get_field("data").unwrap();
    
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    writer.add_document(doc!(
        data_field => tantivy::schema::Value::from(serde_json::json!({
            "title": "Laptop"
        }))
    )).unwrap();
    
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    
    let segment_reader = searcher.segment_reader(0);
    let inv_index = segment_reader.inverted_index(data_field).unwrap();
    
    let mut terms_list = Vec::new();
    let mut term_stream = inv_index.terms().stream().unwrap();
    while let Some((term_bytes, _)) = term_stream.next() {
        let term_str = String::from_utf8_lossy(term_bytes);
        terms_list.push(term_str.to_string());
    }
    
    println!("\n=== INDEXED TERMS ===");
    for term in &terms_list {
        println!("{:?}", term);
    }
    
    let has_corruption = terms_list.iter().any(|t| t.contains("\\0sla") || t.contains("\0sla"));
    let has_correct = terms_list.iter().any(|t| t.contains("\\0la") || t.contains("\0la"));
    
    println!("\nCorruption detected (\\0sla): {}", has_corruption);
    println!("Correct format (\\0la): {}", has_correct);
    
    assert!(!has_corruption, "Terms should not have 's' between path and token");
}

#[test]
fn test_upstream_json_text_default_tokenizer() {
    let mut schema_builder = Schema::builder();
    
    let json_options = JsonObjectOptions::default()
        .set_indexed()
        .set_stored();
    
    schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema.clone());
    let data_field = schema.get_field("data").unwrap();
    
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    
    writer.add_document(doc!(
        data_field => tantivy::schema::Value::from(serde_json::json!({
            "title": "Laptop"
        }))
    )).unwrap();
    
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    
    let segment_reader = searcher.segment_reader(0);
    let inv_index = segment_reader.inverted_index(data_field).unwrap();
    
    let mut terms_list = Vec::new();
    let mut term_stream = inv_index.terms().stream().unwrap();
    while let Some((term_bytes, _)) = term_stream.next() {
        let term_str = String::from_utf8_lossy(term_bytes);
        terms_list.push(term_str.to_string());
    }
    
    println!("\n=== DEFAULT TOKENIZER TERMS ===");
    for term in &terms_list {
        println!("{:?}", term);
    }
    
    assert!(!terms_list.is_empty(), "Should have indexed terms");
}