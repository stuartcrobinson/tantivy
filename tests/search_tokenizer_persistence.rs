use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::{Index, doc};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tempfile::TempDir;

#[test]
fn test_search_tokenizer_persists_across_reload() {
    let temp_dir = TempDir::new().unwrap();
    let index_path = temp_dir.path();
    
    let json_field = {
        let mut schema_builder = Schema::builder();
        
        let text_indexing = TextFieldIndexing::default()
            .set_tokenizer("edge_ngram")
            .set_search_tokenizer("simple")
            .set_index_option(IndexRecordOption::WithFreqsAndPositions);
        
        let json_options = JsonObjectOptions::default()
            .set_indexing_options(text_indexing);
        
        let field = schema_builder.add_json_field("data", json_options);
        let schema = schema_builder.build();
        
        let index = Index::create_in_dir(index_path, schema.clone()).unwrap();
        
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
        writer.add_document(doc!(field => serde_json::json!({"title": "Test"}))).unwrap();
        writer.commit().unwrap();
        
        field
    };
    
    let reloaded_index = Index::open_in_dir(index_path).unwrap();
    
    let edge_ngram_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
        .build();
    reloaded_index.tokenizers().register("edge_ngram", edge_ngram_tokenizer);
    
    let simple_tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .build();
    reloaded_index.tokenizers().register("simple", simple_tokenizer);
    
    let reloaded_schema = reloaded_index.schema();
    let reloaded_field = reloaded_schema.get_field_entry(json_field);
    
    let json_options = match reloaded_field.field_type() {
        tantivy::schema::FieldType::JsonObject(opts) => opts,
        _ => panic!("Expected JSON field"),
    };
    let text_indexing = json_options.get_text_indexing_options().unwrap();
    
    assert_eq!(text_indexing.tokenizer(), "edge_ngram",
        "Index tokenizer should persist");
    assert_eq!(text_indexing.search_tokenizer(), "simple",
        "Search tokenizer should persist across reload");
}

#[test]
fn test_backward_compat_missing_search_tokenizer() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("default");
    
    let json_options = JsonObjectOptions::default()
        .set_indexing_options(text_indexing);
    
    schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema);
    let field = index.schema().get_field("data").unwrap();
    let schema = index.schema();
    let field_entry = schema.get_field_entry(field);
    let json_opts = match field_entry.field_type() {
        tantivy::schema::FieldType::JsonObject(opts) => opts,
        _ => panic!("Expected JSON field"),
    };
    let indexing = json_opts.get_text_indexing_options().unwrap();
    
    assert_eq!(indexing.search_tokenizer(), "default",
        "Old schemas without search_tokenizer should default to index tokenizer");
}