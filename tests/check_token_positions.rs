use tantivy::schema::{JsonObjectOptions, Schema, TextFieldIndexing, IndexRecordOption};
use tantivy::tokenizer::{LowerCaser, TextAnalyzer, WhitespaceTokenizer};
use tantivy::Index;

#[test]
fn test_edge_ngram_token_positions() {
    let mut schema_builder = Schema::builder();
    
    let text_indexing = TextFieldIndexing::default()
        .set_tokenizer("edge_ngram")
        .set_index_option(IndexRecordOption::WithFreqsAndPositions);
    
    let json_options = JsonObjectOptions::default()
        .set_indexing_options(text_indexing);
    
    let _json_field = schema_builder.add_json_field("data", json_options);
    let schema = schema_builder.build();
    
    let index = Index::create_in_ram(schema);
    
    let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
        .filter(LowerCaser)
        .filter(tantivy::tokenizer::EdgeNgramFilter::new(2, 10).unwrap())
        .build();
    
    index.tokenizers().register("edge_ngram", tokenizer.clone());
    
    println!("\n=== TOKENIZATION OUTPUT ===");
    let mut token_stream = tokenizer.token_stream("Gaming Laptop");
    let mut tokens = Vec::new();
    while let Some(token) = token_stream.next() {
        tokens.push(token.clone());
        println!("Token: {:?}, Position: {}, Offset: {:?}", 
                 token.text, token.position, token.offset_from..token.offset_to);
    }
    
    println!("\n=== POSITION ANALYSIS ===");
    for (i, token) in tokens.iter().enumerate() {
        println!("[{}] '{}' at position {}", i, token.text, token.position);
    }
    
    let la_tokens: Vec<_> = tokens.iter().filter(|t| t.text == "la").collect();
    let lap_tokens: Vec<_> = tokens.iter().filter(|t| t.text == "lap").collect();
    
    println!("\n=== PHRASE QUERY ANALYSIS ===");
    println!("'la' tokens: {:?}", la_tokens.iter().map(|t| t.position).collect::<Vec<_>>());
    println!("'lap' tokens: {:?}", lap_tokens.iter().map(|t| t.position).collect::<Vec<_>>());
    
    if let (Some(la), Some(lap)) = (la_tokens.first(), lap_tokens.first()) {
        let consecutive = lap.position == la.position + 1;
        println!("\nAre 'la' and 'lap' consecutive? {}", consecutive);
        println!("This explains why PhraseQuery(slop=0) {}",
                 if consecutive { "MATCHES" } else { "FAILS" });
    }
}
