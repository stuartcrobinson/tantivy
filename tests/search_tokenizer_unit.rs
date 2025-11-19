use tantivy::schema::{TextFieldIndexing, IndexRecordOption};

#[test]
fn test_search_tokenizer_defaults_to_index_tokenizer() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram");
    
    assert_eq!(indexing.search_tokenizer(), "edge_ngram", 
        "When search_tokenizer not set, should return index tokenizer");
}

#[test]
fn test_search_tokenizer_explicit_override() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_search_tokenizer("simple");
    
    assert_eq!(indexing.search_tokenizer(), "simple",
        "Explicit search_tokenizer should override default");
}

#[test]
fn test_search_tokenizer_chain_builder() {
    let indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions)
        .set_search_tokenizer("simple");
    
    assert_eq!(indexing.tokenizer(), "edge_ngram");
    assert_eq!(indexing.search_tokenizer(), "simple");
}