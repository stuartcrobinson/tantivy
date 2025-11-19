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