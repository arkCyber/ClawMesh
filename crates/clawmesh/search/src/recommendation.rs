//! Recommendation engine for intelligent content suggestions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Recommendation type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecommendationType {
    /// Similar content
    Similar,
    /// Trending content
    Trending,
    /// Personalized recommendations
    Personalized,
    /// Popular in community
    Popular,
}

/// Recommendation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Item ID
    pub item_id: i32,
    /// Item type
    pub item_type: String,
    /// Recommendation score
    pub score: f32,
    /// Reason for recommendation
    pub reason: String,
}

/// Recommendation engine
pub struct RecommendationEngine {
    /// User interaction history
    user_history: HashMap<i32, Vec<i32>>,
    /// Item similarity matrix
    similarity_matrix: HashMap<i32, Vec<(i32, f32)>>,
}

impl RecommendationEngine {
    /// Create a new recommendation engine
    #[must_use]
    pub fn new() -> Self {
        Self {
            user_history: HashMap::new(),
            similarity_matrix: HashMap::new(),
        }
    }

    /// Get recommendations for a user
    ///
    /// # Errors
    /// Returns error if recommendation generation fails
    pub fn get_recommendations(
        &self,
        user_id: i32,
        rec_type: RecommendationType,
        limit: usize,
    ) -> Result<Vec<Recommendation>> {
        match rec_type {
            RecommendationType::Similar => self.get_similar_recommendations(user_id, limit),
            RecommendationType::Trending => self.get_trending_recommendations(limit),
            RecommendationType::Personalized => self.get_personalized_recommendations(user_id, limit),
            RecommendationType::Popular => self.get_popular_recommendations(limit),
        }
    }

    /// Get similar content recommendations
    fn get_similar_recommendations(&self, user_id: i32, limit: usize) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();
        
        if let Some(history) = self.user_history.get(&user_id) {
            for &item_id in history.iter().take(5) {
                if let Some(similar_items) = self.similarity_matrix.get(&item_id) {
                    for &(similar_id, score) in similar_items.iter().take(limit) {
                        recommendations.push(Recommendation {
                            item_id: similar_id,
                            item_type: "post".to_string(),
                            score,
                            reason: format!("Similar to item {}", item_id),
                        });
                    }
                }
            }
        }
        
        recommendations.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        recommendations.truncate(limit);
        Ok(recommendations)
    }

    /// Get trending recommendations
    fn get_trending_recommendations(&self, limit: usize) -> Result<Vec<Recommendation>> {
        // TODO: Implement trending algorithm based on recent engagement
        Ok(Vec::new())
    }

    /// Get personalized recommendations
    fn get_personalized_recommendations(&self, user_id: i32, limit: usize) -> Result<Vec<Recommendation>> {
        // TODO: Implement collaborative filtering or content-based filtering
        Ok(Vec::new())
    }

    /// Get popular recommendations
    fn get_popular_recommendations(&self, limit: usize) -> Result<Vec<Recommendation>> {
        // TODO: Implement popularity-based recommendations
        Ok(Vec::new())
    }

    /// Record user interaction
    pub fn record_interaction(&mut self, user_id: i32, item_id: i32) {
        self.user_history
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(item_id);
    }

    /// Update item similarity
    pub fn update_similarity(&mut self, item_id: i32, similar_items: Vec<(i32, f32)>) {
        self.similarity_matrix.insert(item_id, similar_items);
    }

    /// Calculate content similarity using cosine similarity
    ///
    /// # Errors
    /// Returns error if calculation fails
    pub fn calculate_similarity(
        &self,
        features_a: &[f32],
        features_b: &[f32],
    ) -> Result<f32> {
        if features_a.len() != features_b.len() {
            return Err(anyhow::anyhow!("Feature vectors must have same length"));
        }

        let dot_product: f32 = features_a.iter()
            .zip(features_b.iter())
            .map(|(a, b)| a * b)
            .sum();

        let magnitude_a: f32 = features_a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let magnitude_b: f32 = features_b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if magnitude_a == 0.0 || magnitude_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (magnitude_a * magnitude_b))
    }
}

impl Default for RecommendationEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Collaborative filtering recommender
pub struct CollaborativeFilter {
    /// User-item interaction matrix
    interactions: HashMap<i32, HashMap<i32, f32>>,
}

impl CollaborativeFilter {
    /// Create a new collaborative filter
    #[must_use]
    pub fn new() -> Self {
        Self {
            interactions: HashMap::new(),
        }
    }

    /// Add user-item interaction
    pub fn add_interaction(&mut self, user_id: i32, item_id: i32, rating: f32) {
        self.interactions
            .entry(user_id)
            .or_insert_with(HashMap::new)
            .insert(item_id, rating);
    }

    /// Find similar users using Pearson correlation
    ///
    /// # Errors
    /// Returns error if calculation fails
    pub fn find_similar_users(&self, user_id: i32, limit: usize) -> Result<Vec<(i32, f32)>> {
        let user_ratings = match self.interactions.get(&user_id) {
            Some(ratings) => ratings,
            None => return Ok(Vec::new()),
        };

        let mut similarities = Vec::new();

        for (&other_user_id, other_ratings) in &self.interactions {
            if other_user_id == user_id {
                continue;
            }

            let similarity = self.pearson_correlation(user_ratings, other_ratings)?;
            if similarity > 0.0 {
                similarities.push((other_user_id, similarity));
            }
        }

        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        similarities.truncate(limit);
        Ok(similarities)
    }

    /// Calculate Pearson correlation coefficient
    fn pearson_correlation(
        &self,
        ratings_a: &HashMap<i32, f32>,
        ratings_b: &HashMap<i32, f32>,
    ) -> Result<f32> {
        let common_items: Vec<i32> = ratings_a
            .keys()
            .filter(|k| ratings_b.contains_key(k))
            .copied()
            .collect();

        if common_items.len() < 2 {
            return Ok(0.0);
        }

        let mean_a: f32 = common_items.iter().map(|i| ratings_a[i]).sum::<f32>() / common_items.len() as f32;
        let mean_b: f32 = common_items.iter().map(|i| ratings_b[i]).sum::<f32>() / common_items.len() as f32;

        let mut numerator = 0.0;
        let mut sum_sq_a = 0.0;
        let mut sum_sq_b = 0.0;

        for item_id in common_items {
            let diff_a = ratings_a[&item_id] - mean_a;
            let diff_b = ratings_b[&item_id] - mean_b;
            numerator += diff_a * diff_b;
            sum_sq_a += diff_a * diff_a;
            sum_sq_b += diff_b * diff_b;
        }

        if sum_sq_a == 0.0 || sum_sq_b == 0.0 {
            return Ok(0.0);
        }

        Ok(numerator / (sum_sq_a.sqrt() * sum_sq_b.sqrt()))
    }
}

impl Default for CollaborativeFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_engine() {
        let mut engine = RecommendationEngine::new();
        engine.record_interaction(1, 100);
        engine.record_interaction(1, 101);
        
        assert!(engine.user_history.contains_key(&1));
    }

    #[test]
    fn test_cosine_similarity() {
        let engine = RecommendationEngine::new();
        let features_a = vec![1.0, 2.0, 3.0];
        let features_b = vec![1.0, 2.0, 3.0];
        
        let similarity = engine.calculate_similarity(&features_a, &features_b)
            .expect("Failed to calculate similarity");
        assert!((similarity - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_collaborative_filter() {
        let mut filter = CollaborativeFilter::new();
        filter.add_interaction(1, 100, 5.0);
        filter.add_interaction(1, 101, 4.0);
        filter.add_interaction(1, 102, 3.0);
        filter.add_interaction(2, 100, 5.0);
        filter.add_interaction(2, 101, 4.0);
        filter.add_interaction(2, 102, 3.0);
        
        let similar_users = filter.find_similar_users(1, 5)
            .expect("Failed to find similar users");
        assert!(!similar_users.is_empty());
        assert_eq!(similar_users[0].0, 2);
    }

    #[test]
    fn test_pearson_correlation() {
        let mut filter = CollaborativeFilter::new();
        filter.add_interaction(1, 100, 5.0);
        filter.add_interaction(1, 101, 4.0);
        filter.add_interaction(2, 100, 5.0);
        filter.add_interaction(2, 101, 4.0);
        
        let ratings_a = filter.interactions.get(&1).unwrap();
        let ratings_b = filter.interactions.get(&2).unwrap();
        
        let correlation = filter.pearson_correlation(ratings_a, ratings_b)
            .expect("Failed to calculate correlation");
        assert!((correlation - 1.0).abs() < 0.001);
    }
}
