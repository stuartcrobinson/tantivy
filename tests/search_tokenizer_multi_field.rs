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