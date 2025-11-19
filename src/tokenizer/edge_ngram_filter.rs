use super::{Token, TokenFilter, TokenStream, Tokenizer};

pub struct EdgeNgramFilter {
    min_gram: usize,
    max_gram: usize,
}

impl EdgeNgramFilter {
    pub fn new(min_gram: usize, max_gram: usize) -> crate::Result<Self> {
        if min_gram == 0 {
            return Err(crate::TantivyError::InvalidArgument(
                "min_gram must be greater than 0".to_string(),
            ));
        }
        if min_gram > max_gram {
            return Err(crate::TantivyError::InvalidArgument(
                "min_gram must not be greater than max_gram".to_string(),
            ));
        }
        Ok(EdgeNgramFilter { min_gram, max_gram })
    }
}

impl TokenFilter for EdgeNgramFilter {
    type Tokenizer<T: Tokenizer> = EdgeNgramFilterWrapper<T>;

    fn transform<T: Tokenizer>(self, tokenizer: T) -> EdgeNgramFilterWrapper<T> {
        EdgeNgramFilterWrapper {
            inner: tokenizer,
            min_gram: self.min_gram,
            max_gram: self.max_gram,
            ngrams: Vec::new(),
        }
    }
}

pub struct EdgeNgramFilterWrapper<T> {
    inner: T,
    min_gram: usize,
    max_gram: usize,
    ngrams: Vec<Token>,
}

impl<T: Tokenizer> Tokenizer for EdgeNgramFilterWrapper<T> {
    type TokenStream<'a> = EdgeNgramTokenStream<'a, T::TokenStream<'a>>;

    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        self.ngrams.clear();
        EdgeNgramTokenStream {
            tail: self.inner.token_stream(text),
            min_gram: self.min_gram,
            max_gram: self.max_gram,
            ngrams: &mut self.ngrams,
        }
    }
}

pub struct EdgeNgramTokenStream<'a, T> {
    tail: T,
    min_gram: usize,
    max_gram: usize,
    ngrams: &'a mut Vec<Token>,
}

impl<T: TokenStream> EdgeNgramTokenStream<'_, T> {
    fn generate_ngrams(&mut self) {
        let token = self.tail.token();
        let text = token.text.as_str();
        
        self.ngrams.clear();
        
        let char_count = text.chars().count();
        let max_len = self.max_gram.min(char_count);
        
        for len in (self.min_gram..=max_len).rev() {
            let ngram_text: String = text.chars().take(len).collect();
            self.ngrams.push(Token {
                text: ngram_text,
                ..*token
            });
        }
    }
}

impl<T: TokenStream> TokenStream for EdgeNgramTokenStream<'_, T> {
    fn advance(&mut self) -> bool {
        self.ngrams.pop();
        
        if !self.ngrams.is_empty() {
            return true;
        }
        
        if !self.tail.advance() {
            return false;
        }
        
        self.generate_ngrams();
        true
    }
    
    fn token(&self) -> &Token {
        self.ngrams.last().unwrap_or_else(|| self.tail.token())
    }
    
    fn token_mut(&mut self) -> &mut Token {
        self.ngrams.last_mut().unwrap_or_else(|| self.tail.token_mut())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::{TextAnalyzer, WhitespaceTokenizer, LowerCaser};

    #[test]
    fn test_edge_ngram_simple() {
        let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .filter(EdgeNgramFilter::new(2, 5).unwrap())
            .build();
        
        let mut stream = tokenizer.token_stream("gaming");
        assert_eq!(stream.next().unwrap().text, "ga");
        assert_eq!(stream.next().unwrap().text, "gam");
        assert_eq!(stream.next().unwrap().text, "gami");
        assert_eq!(stream.next().unwrap().text, "gamin");
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_edge_ngram_multi_word() {
        let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .filter(EdgeNgramFilter::new(2, 6).unwrap())
            .build();
        
        let mut stream = tokenizer.token_stream("gaming laptop");
        assert_eq!(stream.next().unwrap().text, "ga");
        assert_eq!(stream.next().unwrap().text, "gam");
        assert_eq!(stream.next().unwrap().text, "gami");
        assert_eq!(stream.next().unwrap().text, "gamin");
        assert_eq!(stream.next().unwrap().text, "gaming");
        assert_eq!(stream.next().unwrap().text, "la");
        assert_eq!(stream.next().unwrap().text, "lap");
        assert_eq!(stream.next().unwrap().text, "lapt");
        assert_eq!(stream.next().unwrap().text, "lapto");
        assert_eq!(stream.next().unwrap().text, "laptop");
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_edge_ngram_utf8() {
        let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .filter(EdgeNgramFilter::new(2, 4).unwrap())
            .build();
        
        let mut stream = tokenizer.token_stream("café");
        assert_eq!(stream.next().unwrap().text, "ca");
        assert_eq!(stream.next().unwrap().text, "caf");
        assert_eq!(stream.next().unwrap().text, "café");
        assert_eq!(stream.next(), None);
    }

    #[test]
    fn test_edge_ngram_short_word() {
        let mut tokenizer = TextAnalyzer::builder(WhitespaceTokenizer::default())
            .filter(LowerCaser)
            .filter(EdgeNgramFilter::new(2, 10).unwrap())
            .build();
        
        let mut stream = tokenizer.token_stream("go");
        assert_eq!(stream.next().unwrap().text, "go");
        assert_eq!(stream.next(), None);
    }

    #[test]
    #[should_panic(expected = "min_gram must be greater than 0")]
    fn test_edge_ngram_zero_min() {
        EdgeNgramFilter::new(0, 5).unwrap();
    }

    #[test]
    #[should_panic(expected = "min_gram must not be greater than max_gram")]
    fn test_edge_ngram_invalid_range() {
        EdgeNgramFilter::new(5, 2).unwrap();
    }
}