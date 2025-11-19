use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption, Term};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};

#[test]
fn test_compare_indexed_vs_query_term_bytes() {
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
    
    let segment_reader = searcher.segment_reader(0);
    let inv_index = segment_reader.inverted_index(json_field).unwrap();
    
    println!("\n=== INDEXED TERMS ===");
    let mut term_stream = inv_index.terms().stream().unwrap();
    while let Some((term_bytes, _)) = term_stream.next() {
        println!("Hex: {:02x?}", term_bytes);
        println!("UTF8 lossy: {:?}", String::from_utf8_lossy(term_bytes));
    }
    
    println!("\n=== QUERY TERM CONSTRUCTION ===");
    let mut query_term = Term::from_field_json_path(json_field, "title", false);
    query_term.append_type_and_str("lap");
    
    let query_bytes = query_term.serialized_term();
    println!("Query term bytes: {:02x?}", query_bytes);
    println!("Query term UTF8: {:?}", String::from_utf8_lossy(query_bytes));
    
    println!("\n=== COMPARISON ===");
    println!("Indexed 'lap' term: [74, 69, 74, 6c, 65, 00, 73, 6c, 61, 70]");
    println!("Query term:         {:02x?}", query_bytes);
    
    let term_info = inv_index.terms().get(query_bytes);
    println!("\nDirect term lookup: {}", if term_info.is_ok() { "FOUND" } else { "NOT FOUND" });
}
