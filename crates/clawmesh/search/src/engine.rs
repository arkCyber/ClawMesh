//! Search engine implementation

use crate::{SearchConfig, SearchQuery, SearchResult};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Search engine
pub struct SearchEngine {
    config: SearchConfig,
    index: Arc<RwLock<SearchIndex>>,
}

/// Search index for fast lookups
pub struct SearchIndex {
    /// Inverted index: term -> document IDs
    inverted_index: HashMap<String, Vec<i32>>,
    /// Document store: document ID -> content
    documents: HashMap<i32, String>,
    /// Term frequencies
    term_frequencies: HashMap<String, usize>,
}

impl SearchEngine {
    /// Create a new search engine
    #[must_use]
    pub fn new(config: SearchConfig) -> Self {
        Self {
            config,
            index: Arc::new(RwLock::new(SearchIndex::new())),
        }
    }

    /// Search with a query
    ///
    /// # Errors
    /// Returns error if search fails
    pub async fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>> {
        query.validate()?;
        
        // TODO: Implement actual search logic
        // 1. Tokenize query
        // 2. Look up terms in inverted index
        // 3. Score and rank results
        // 4. Apply filters
        // 5. Sort and paginate
        
        Ok(Vec::new())
    }

    /// Index a document
    ///
    /// # Errors
    /// Returns error if indexing fails
    pub fn index_document(&self, id: i32, content: &str) -> Result<()> {
        let mut index = self.index.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        index.add_document(id, content)
    }

    /// Remove a document from index
    ///
    /// # Errors
    /// Returns error if removal fails
    pub fn remove_document(&self, id: i32) -> Result<()> {
        let mut index = self.index.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        index.remove_document(id)
    }

    /// Get index statistics
    ///
    /// # Errors
    /// Returns error if lock is poisoned
    pub fn get_stats(&self) -> Result<IndexStats> {
        let index = self.index.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        Ok(IndexStats {
            document_count: index.documents.len(),
            term_count: index.inverted_index.len(),
            total_terms: index.term_frequencies.values().sum(),
        })
    }
}

impl SearchIndex {
    /// Create a new search index
    #[must_use]
    pub fn new() -> Self {
        Self {
            inverted_index: HashMap::new(),
            documents: HashMap::new(),
            term_frequencies: HashMap::new(),
        }
    }

    /// Add a document to the index
    ///
    /// # Errors
    /// Returns error if indexing fails
    pub fn add_document(&mut self, id: i32, content: &str) -> Result<()> {
        // Tokenize content
        let tokens = Self::tokenize(content);
        
        // Store document
        self.documents.insert(id, content.to_string());
        
        // Update inverted index
        for token in &tokens {
            self.inverted_index
                .entry(token.clone())
                .or_insert_with(Vec::new)
                .push(id);
            
            *self.term_frequencies.entry(token.clone()).or_insert(0) += 1;
        }
        
        Ok(())
    }

    /// Remove a document from the index
    ///
    /// # Errors
    /// Returns error if removal fails
    pub fn remove_document(&mut self, id: i32) -> Result<()> {
        if let Some(content) = self.documents.remove(&id) {
            let tokens = Self::tokenize(&content);
            
            for token in &tokens {
                if let Some(doc_ids) = self.inverted_index.get_mut(token) {
                    doc_ids.retain(|&doc_id| doc_id != id);
                }
                
                if let Some(freq) = self.term_frequencies.get_mut(token) {
                    *freq = freq.saturating_sub(1);
                }
            }
        }
        
        Ok(())
    }

    /// Tokenize text into terms
    fn tokenize(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.trim_matches(|c: char| !c.is_alphanumeric()))
            .filter(|s| !s.is_empty() && s.len() > 2)
            .map(String::from)
            .collect()
    }

    /// Search for documents containing a term
    #[must_use]
    pub fn search_term(&self, term: &str) -> Vec<i32> {
        self.inverted_index
            .get(&term.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }
}

impl Default for SearchIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Index statistics
#[derive(Debug, Clone)]
pub struct IndexStats {
    /// Number of documents
    pub document_count: usize,
    /// Number of unique terms
    pub term_count: usize,
    /// Total number of terms
    pub total_terms: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let tokens = SearchIndex::tokenize("Hello, World! This is a test.");
        assert!(tokens.contains(&"hello".to_string()));
        assert!(tokens.contains(&"world".to_string()));
        assert!(tokens.contains(&"test".to_string()));
        assert!(!tokens.contains(&"is".to_string())); // Too short
    }

    #[test]
    fn test_index_document() {
        let mut index = SearchIndex::new();
        index.add_document(1, "hello world").expect("Failed to index");
        
        let results = index.search_term("hello");
        assert_eq!(results, vec![1]);
    }

    #[test]
    fn test_remove_document() {
        let mut index = SearchIndex::new();
        index.add_document(1, "hello world").expect("Failed to index");
        index.remove_document(1).expect("Failed to remove");
        
        let results = index.search_term("hello");
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_engine_stats() {
        let engine = SearchEngine::new(SearchConfig::default());
        engine.index_document(1, "hello world").expect("Failed to index");
        
        let stats = engine.get_stats().expect("Failed to get stats");
        assert_eq!(stats.document_count, 1);
        assert!(stats.term_count > 0);
    }
}
