//! Search query parsing and filtering

use crate::ResultType;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Search query builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Search text
    pub text: String,
    /// Result types to search
    pub result_types: Vec<ResultType>,
    /// Filters to apply
    pub filters: Vec<SearchFilter>,
    /// Sort order
    pub sort: SortOrder,
    /// Page number
    pub page: usize,
    /// Results per page
    pub limit: usize,
}

/// Search filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchFilter {
    /// Filter by author ID
    Author(i32),
    /// Filter by community ID
    Community(i32),
    /// Filter by date range
    DateRange {
        /// Start date
        from: DateTime<Utc>,
        /// End date
        to: DateTime<Utc>,
    },
    /// Filter by score threshold
    MinScore(i32),
    /// Filter by language
    Language(String),
    /// Filter by tags
    Tags(Vec<String>),
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SortOrder {
    /// Sort by relevance score
    Relevance,
    /// Sort by creation date (newest first)
    Newest,
    /// Sort by creation date (oldest first)
    Oldest,
    /// Sort by score (highest first)
    TopScore,
    /// Sort by comment count
    MostComments,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: i32,
    /// Result type
    pub result_type: ResultType,
    /// Title or name
    pub title: String,
    /// Content snippet
    pub snippet: String,
    /// Relevance score
    pub score: f32,
    /// Author ID
    pub author_id: i32,
    /// Author name
    pub author_name: String,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

impl SearchQuery {
    /// Create a new search query
    #[must_use]
    pub fn new(text: String) -> Self {
        Self {
            text,
            result_types: vec![
                ResultType::Post,
                ResultType::Comment,
                ResultType::Community,
                ResultType::User,
            ],
            filters: Vec::new(),
            sort: SortOrder::Relevance,
            page: 1,
            limit: 20,
        }
    }

    /// Add a filter to the query
    pub fn with_filter(mut self, filter: SearchFilter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Set result types
    pub fn with_types(mut self, types: Vec<ResultType>) -> Self {
        self.result_types = types;
        self
    }

    /// Set sort order
    pub fn with_sort(mut self, sort: SortOrder) -> Self {
        self.sort = sort;
        self
    }

    /// Set pagination
    pub fn with_pagination(mut self, page: usize, limit: usize) -> Self {
        self.page = page;
        self.limit = limit;
        self
    }

    /// Execute the search query
    ///
    /// # Errors
    /// Returns error if search execution fails
    pub async fn execute(&self) -> Result<Vec<SearchResult>> {
        // TODO: Implement actual search execution
        Err(anyhow::anyhow!("Not implemented"))
    }

    /// Validate the query
    ///
    /// # Errors
    /// Returns error if query is invalid
    pub fn validate(&self) -> Result<()> {
        if self.text.is_empty() {
            return Err(anyhow::anyhow!("Search text cannot be empty"));
        }
        if self.text.len() > 500 {
            return Err(anyhow::anyhow!("Search text too long (max 500 characters)"));
        }
        if self.limit > 100 {
            return Err(anyhow::anyhow!("Limit too large (max 100)"));
        }
        Ok(())
    }
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self::new(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("test query".to_string())
            .with_sort(SortOrder::Newest)
            .with_pagination(2, 30);
        
        assert_eq!(query.text, "test query");
        assert_eq!(query.sort, SortOrder::Newest);
        assert_eq!(query.page, 2);
        assert_eq!(query.limit, 30);
    }

    #[test]
    fn test_query_validation() {
        let query = SearchQuery::new("valid query".to_string());
        assert!(query.validate().is_ok());

        let empty_query = SearchQuery::new(String::new());
        assert!(empty_query.validate().is_err());

        let long_query = SearchQuery::new("a".repeat(501));
        assert!(long_query.validate().is_err());
    }

    #[test]
    fn test_with_filter() {
        let query = SearchQuery::new("test".to_string())
            .with_filter(SearchFilter::Author(123))
            .with_filter(SearchFilter::MinScore(10));
        
        assert_eq!(query.filters.len(), 2);
    }
}
