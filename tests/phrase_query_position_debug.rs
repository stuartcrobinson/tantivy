use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};
use tantivy::query::PhraseQuery;
use tantivy::schema::Term;
use tantivy::collector::TopDocs;

#[test]
fn test_phrase_query_position_matching() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let json_options = JsonObjectOptions::default()
        .set_indexing_options(text_indexing);
    
    let json_field = schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema);
    
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
    
    println!("\n=== MANUAL PHRASE QUERY ===");
    let mut term_la = Term::from_field_json_path(json_field, "title", false);
    term_la.append_type_and_str("la");
    
    let mut term_lap = Term::from_field_json_path(json_field, "title", false);
    term_lap.append_type_and_str("lap");
    
    let phrase_terms = vec![(0, term_la), (1, term_lap)];
    let phrase_query = PhraseQuery::new_with_offset(phrase_terms);
    
    println!("Query: PhraseQuery with offsets [(0, 'la'), (1, 'lap')]");
    println!("Expected: 'la' at relative pos 0, 'lap' at relative pos 1");
    
    let results = searcher.search(&phrase_query, &TopDocs::with_limit(10)).unwrap();
    println!("Hits: {}", results.len());
    
    println!("\n=== SAME POSITION PHRASE QUERY ===");
    let mut term_la_same = Term::from_field_json_path(json_field, "title", false);
    term_la_same.append_type_and_str("la");
    let mut term_lap_same = Term::from_field_json_path(json_field, "title", false);
    term_lap_same.append_type_and_str("lap");
    let phrase_terms_same = vec![(0, term_la_same), (0, term_lap_same)];
    let phrase_query_same = PhraseQuery::new_with_offset(phrase_terms_same);
    
    println!("Query: PhraseQuery with offsets [(0, 'la'), (0, 'lap')]");
    println!("Expected: Both at same relative position");
    
    let results_same = searcher.search(&phrase_query_same, &TopDocs::with_limit(10)).unwrap();
    println!("Hits: {}", results_same.len());
    
    println!("\n=== ANALYSIS ===");
    println!("If offset (0,1) matches: PhraseQuery incorrectly matches different positions");
    println!("If offset (0,0) matches: EdgeNgramFilter position assignment is wrong");
}
