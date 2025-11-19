use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption, STORED};
use tantivy::tokenizer::{LowerCaser, NgramTokenizer, TextAnalyzer, WhitespaceTokenizer};
use tantivy::{doc, Index, IndexWriter};
use tantivy::query::TermQuery;
use tantivy::schema::Term;
use tantivy::collector::TopDocs;

#[test]
fn test_edge_ngram_indexing_works() {
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
        .filter(NgramTokenizer::prefix_only(2, 10).unwrap())
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
    
    let mut term = Term::from_field_json_path(json_field, "title", false);
    term.append_type_and_str("la");
    
    let term_info = inv_index.get_term_info(&term).unwrap();
    assert!(term_info.is_some(), "Edge ngram 'la' should be indexed");
    
    term = Term::from_field_json_path(json_field, "title", false);
    term.append_type_and_str("lap");
    let term_info = inv_index.get_term_info(&term).unwrap();
    assert!(term_info.is_some(), "Edge ngram 'lap' should be indexed");
    
    term = Term::from_field_json_path(json_field, "title", false);
    term.append_type_and_str("lapt");
    let term_info = inv_index.get_term_info(&term).unwrap();
    assert!(term_info.is_some(), "Edge ngram 'lapt' should be indexed");
    
    term = Term::from_field_json_path(json_field, "title", false);
    term.append_type_and_str("laptop");
    let term_info = inv_index.get_term_info(&term).unwrap();
    assert!(term_info.is_some(), "Full word 'laptop' should be indexed");
}

#[test]
fn test_edge_ngram_manual_term_query_works() {
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
        .filter(NgramTokenizer::prefix_only(2, 10).unwrap())
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
    
    assert_eq!(results.len(), 1, "Manual TermQuery with 'lap' should match 'Gaming Laptop'");
}

#[test]
fn test_edge_ngram_query_parser_creates_phrase_query() {
    use tantivy::query::QueryParser;
    
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
        .filter(NgramTokenizer::prefix_only(2, 10).unwrap())
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
    let results = searcher.search(&query, &TopDocs::with_limit(10)).unwrap();
    
    assert_eq!(results.len(), 0, "QueryParser should create PhraseQuery that fails (proves the bug)");
}