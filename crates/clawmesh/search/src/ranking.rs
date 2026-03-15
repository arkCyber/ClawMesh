//! Search result ranking algorithms

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Ranking algorithm type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RankingAlgorithm {
    /// TF-IDF (Term Frequency-Inverse Document Frequency)
    TfIdf,
    /// BM25 (Best Matching 25)
    Bm25,
    /// Simple term frequency
    TermFrequency,
    /// Hybrid (combines multiple algorithms)
    Hybrid,
}

/// Search score for a result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchScore {
    /// Overall relevance score
    pub relevance: f32,
    /// Freshness score (based on recency)
    pub freshness: f32,
    /// Popularity score (based on votes/comments)
    pub popularity: f32,
    /// Combined final score
    pub final_score: f32,
}

impl SearchScore {
    /// Create a new search score
    #[must_use]
    pub fn new(relevance: f32, freshness: f32, popularity: f32) -> Self {
        let final_score = Self::calculate_final_score(relevance, freshness, popularity);
        Self {
            relevance,
            freshness,
            popularity,
            final_score,
        }
    }

    /// Calculate final score using weighted combination
    fn calculate_final_score(relevance: f32, freshness: f32, popularity: f32) -> f32 {
        const RELEVANCE_WEIGHT: f32 = 0.6;
        const FRESHNESS_WEIGHT: f32 = 0.2;
        const POPULARITY_WEIGHT: f32 = 0.2;

        relevance * RELEVANCE_WEIGHT
            + freshness * FRESHNESS_WEIGHT
            + popularity * POPULARITY_WEIGHT
    }
}

/// TF-IDF scorer
pub struct TfIdfScorer {
    /// Total number of documents
    total_docs: usize,
}

impl TfIdfScorer {
    /// Create a new TF-IDF scorer
    #[must_use]
    pub fn new(total_docs: usize) -> Self {
        Self { total_docs }
    }

    /// Calculate TF-IDF score
    ///
    /// # Errors
    /// Returns error if calculation fails
    pub fn score(
        &self,
        term_freq: usize,
        doc_freq: usize,
        doc_length: usize,
    ) -> Result<f32> {
        if doc_freq == 0 || doc_length == 0 {
            return Ok(0.0);
        }

        let tf = term_freq as f32 / doc_length as f32;
        let idf = ((self.total_docs as f32) / (doc_freq as f32)).ln();
        
        Ok(tf * idf)
    }
}

/// BM25 scorer
pub struct Bm25Scorer {
    /// Total number of documents
    total_docs: usize,
    /// Average document length
    avg_doc_length: f32,
    /// k1 parameter (term saturation)
    k1: f32,
    /// b parameter (length normalization)
    b: f32,
}

impl Bm25Scorer {
    /// Create a new BM25 scorer with default parameters
    #[must_use]
    pub fn new(total_docs: usize, avg_doc_length: f32) -> Self {
        Self {
            total_docs,
            avg_doc_length,
            k1: 1.5,
            b: 0.75,
        }
    }

    /// Calculate BM25 score
    ///
    /// # Errors
    /// Returns error if calculation fails
    pub fn score(
        &self,
        term_freq: usize,
        doc_freq: usize,
        doc_length: usize,
    ) -> Result<f32> {
        if doc_freq == 0 || doc_length == 0 {
            return Ok(0.0);
        }

        let tf = term_freq as f32;
        let df = doc_freq as f32;
        let dl = doc_length as f32;

        // IDF component
        let idf = ((self.total_docs as f32 - df + 0.5) / (df + 0.5)).ln();

        // TF component with length normalization
        let norm = 1.0 - self.b + self.b * (dl / self.avg_doc_length);
        let tf_component = (tf * (self.k1 + 1.0)) / (tf + self.k1 * norm);

        Ok(idf * tf_component)
    }
}

/// Calculate freshness score based on age
#[must_use]
pub fn calculate_freshness_score(age_days: f32) -> f32 {
    // Exponential decay: score = e^(-age/30)
    // Half-life of ~30 days
    const DECAY_RATE: f32 = 30.0;
    (-age_days / DECAY_RATE).exp()
}

/// Calculate popularity score based on votes and comments
#[must_use]
pub fn calculate_popularity_score(upvotes: i32, downvotes: i32, comments: i32) -> f32 {
    let score = upvotes - downvotes;
    let total_engagement = upvotes + downvotes + comments;
    
    if total_engagement == 0 {
        return 0.0;
    }

    // Wilson score confidence interval
    let n = total_engagement as f32;
    let p = (upvotes as f32) / n;
    
    // Simplified Wilson score
    let z = 1.96; // 95% confidence
    let denominator = 1.0 + z * z / n;
    let numerator = p + z * z / (2.0 * n) - z * ((p * (1.0 - p) + z * z / (4.0 * n)) / n).sqrt();
    
    (numerator / denominator).max(0.0).min(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_score() {
        let score = SearchScore::new(0.8, 0.6, 0.7);
        assert!(score.final_score > 0.0);
        assert!(score.final_score <= 1.0);
    }

    #[test]
    fn test_tfidf_scorer() {
        let scorer = TfIdfScorer::new(1000);
        let score = scorer.score(5, 10, 100).expect("Failed to score");
        assert!(score > 0.0);
    }

    #[test]
    fn test_bm25_scorer() {
        let scorer = Bm25Scorer::new(1000, 100.0);
        let score = scorer.score(5, 10, 100).expect("Failed to score");
        assert!(score > 0.0);
    }

    #[test]
    fn test_freshness_score() {
        let score_new = calculate_freshness_score(1.0);
        let score_old = calculate_freshness_score(100.0);
        assert!(score_new > score_old);
        assert!(score_new <= 1.0);
    }

    #[test]
    fn test_popularity_score() {
        let score = calculate_popularity_score(100, 10, 50);
        assert!(score > 0.0);
        assert!(score <= 1.0);
    }

    #[test]
    fn test_zero_engagement() {
        let score = calculate_popularity_score(0, 0, 0);
        assert_eq!(score, 0.0);
    }
}
