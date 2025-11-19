use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;

#[test]
fn test_queryparser_phrase_query_on_edge_ngrams() {
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
    
    let mut writer: IndexWriter = index.writer(50_000_000).unwrap();
    let doc = doc!(json_field => serde_json::json!({"title": "Gaming Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    let query = query_parser.parse_query("data.title:lap").unwrap();
    
    println!("\n=== QUERY TYPE ===");
    println!("{:?}", query);
    
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    println!("\n=== EXPECTED BEHAVIOR ===");
    println!("QueryParser tokenizes 'lap' → ['la', 'lap']");
    println!("Creates PhraseQuery requiring consecutive terms");
    println!("In 'Gaming Laptop': 'la' and 'lap' NOT consecutive");
    println!("Expected hits: 0");
    println!("Actual hits: {}", results.len());
    
    // This SHOULD fail if QueryParser creates PhraseQuery
    // If it passes, we have bigger questions about how matching works
    assert_eq!(results.len(), 0, "PhraseQuery should fail on non-consecutive edge ngrams");
}
