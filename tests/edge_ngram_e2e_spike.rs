use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};
use tantivy::query::TermQuery;
use tantivy::schema::Term;
use tantivy::collector::TopDocs;

#[test]
fn test_edge_ngram_indexing_correct_format() {
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
    
    let has_corruption = terms_list.iter().any(|t| {
        t.contains("\\0sla") || t.contains("\0sla") || 
        t.contains("\\0slap") || t.contains("\0slap")
    });
    
    assert!(has_corruption, "Terms should have 's' type byte (Type::Str indicator)");
    
    let has_ngrams = terms_list.iter().any(|t| 
        t.contains("sla") || t.contains("slap") || t.contains("slapt")
    );
    assert!(has_ngrams, "Should have ngram terms with Type::Str byte ('s') before token text");
}

#[test]
fn test_edge_ngram_manual_term_query() {
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
    
    let mut term = Term::from_field_json_path(json_field, "title", false);
    term.append_type_and_str("lap");
    
    let query = TermQuery::new(term, IndexRecordOption::Basic);
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    println!("\nManual TermQuery for 'lap': {} hits", results.len());
    assert_eq!(results.len(), 1, "Manual TermQuery should find 'lap' in 'Gaming Laptop'");
}

#[test]
fn test_edge_ngram_multi_word_tokens() {
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
    
    let mut term_gam = Term::from_field_json_path(json_field, "title", false);
    term_gam.append_type_and_str("gam");
    let query_gam = TermQuery::new(term_gam, IndexRecordOption::Basic);
    let results_gam = searcher.search(&query_gam, &TopDocs::with_limit(10)).unwrap();
    
    let mut term_lap = Term::from_field_json_path(json_field, "title", false);
    term_lap.append_type_and_str("lap");
    let query_lap = TermQuery::new(term_lap, IndexRecordOption::Basic);
    let results_lap = searcher.search(&query_lap, &TopDocs::with_limit(10)).unwrap();
    
    println!("\nQuery 'gam': {} hits", results_gam.len());
    println!("Query 'lap': {} hits", results_lap.len());
    
    assert_eq!(results_gam.len(), 1, "Should match 'Gaming' prefix");
    assert_eq!(results_lap.len(), 1, "Should match 'Laptop' prefix");
}