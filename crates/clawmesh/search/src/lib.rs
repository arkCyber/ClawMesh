//! ClawMesh Advanced Search System
//! 
//! Provides full-text search, intelligent recommendations, and advanced filtering

pub mod engine;
pub mod query;
pub mod ranking;
pub mod recommendation;

pub use engine::{SearchEngine, SearchIndex};
pub use query::{SearchQuery, SearchFilter, SearchResult};
pub use ranking::{RankingAlgorithm, SearchScore};
pub use recommendation::{RecommendationEngine, RecommendationType};

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Search configuration
#[derive(Debug, Clone)]
pub struct SearchConfig {
    /// Maximum results per page
    pub max_results: usize,
    /// Enable fuzzy matching
    pub enable_fuzzy: bool,
    /// Minimum search score threshold
    pub min_score: f32,
    /// Enable recommendations
    pub enable_recommendations: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            max_results: 50,
            enable_fuzzy: true,
            min_score: 0.1,
            enable_recommendations: true,
        }
    }
}

/// Search result type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResultType {
    /// Post result
    Post,
    /// Comment result
    Comment,
    /// Community result
    Community,
    /// User result
    User,
    /// Group message result
    GroupMessage,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SearchConfig::default();
        assert_eq!(config.max_results, 50);
        assert!(config.enable_fuzzy);
    }

    #[test]
    fn test_result_type() {
        let result_type = ResultType::Post;
        assert_eq!(result_type, ResultType::Post);
    }
}
