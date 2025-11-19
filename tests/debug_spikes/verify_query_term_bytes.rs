use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};
use tantivy::query::QueryParser;
use tantivy::collector::TopDocs;

#[test]
fn test_query_parser_term_construction() {
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
    let doc = doc!(json_field => serde_json::json!({"title": "Laptop"}));
    writer.add_document(doc).unwrap();
    writer.commit().unwrap();
    
    let reader = index.reader().unwrap();
    let searcher = reader.searcher();
    
    // Get actual indexed terms
    let segment_reader = searcher.segment_reader(0);
    let inv_index = segment_reader.inverted_index(json_field).unwrap();
    let mut indexed_terms = Vec::new();
    let mut term_stream = inv_index.terms().stream().unwrap();
    while let Some((term_bytes, _)) = term_stream.next() {
        indexed_terms.push(term_bytes.to_vec());
    }
    
    println!("\n=== INDEXED TERMS (hex) ===");
    for term in &indexed_terms {
        println!("{:02x?}", term);
    }
    
    // Now parse a query and inspect what term bytes QueryParser constructs
    let query_parser = QueryParser::for_index(&index, vec![json_field]);
    let query = query_parser.parse_query("data.title:lap").unwrap();
    
    println!("\n=== QUERY STRUCTURE ===");
    println!("{:?}", query);
    
    // Search and see if it matches
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    println!("\n=== RESULTS ===");
    println!("Hits: {}", results.len());
    
    // CRITICAL: If hits = 1, QueryParser IS constructing corrupted terms
    // If hits = 0, QueryParser constructs correct terms (proof corruption breaks queries)
    assert_eq!(results.len(), 1, "QueryParser should match despite corruption");
}
