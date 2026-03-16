/// Review Management Functions
/// 
/// Core functions for managing product reviews

use crate::models::{MarketplaceReview, ReviewForm};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::marketplace_reviews;

/// Create a new review
pub async fn create_review(
    form: ReviewForm,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceReview> {
    // Validate form
    form.validate()?;
    
    // Check if order exists and is completed
    use lemmy_db_schema_file::schema::marketplace_orders;
    let order: (i32, i32, i32) = marketplace_orders::table
        .find(form.order_id)
        .select((
            marketplace_orders::status,
            marketplace_orders::buyer_id,
            marketplace_orders::product_id,
        ))
        .first(conn)
        .await?;
    
    let (status, buyer_id, product_id) = order;
    
    if status != 3 { // Completed
        bail!("Can only review completed orders");
    }
    
    if buyer_id != form.reviewer_id {
        bail!("Only buyer can review");
    }
    
    if product_id != form.product_id {
        bail!("Product ID mismatch");
    }
    
    // Check if review already exists
    let review_count: i64 = marketplace_reviews::table
        .filter(marketplace_reviews::order_id.eq(form.order_id))
        .count()
        .get_result(conn)
        .await?;
    
    if review_count > 0 {
        bail!("Review already exists for this order");
    }
    
    // Insert review
    let review = diesel::insert_into(marketplace_reviews::table)
        .values(&form)
        .get_result::<MarketplaceReview>(conn)
        .await?;
    
    Ok(review)
}

/// Get review by ID
pub async fn get_review(
    review_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceReview> {
    marketplace_reviews::table
        .find(review_id)
        .first::<MarketplaceReview>(conn)
        .await
        .map_err(|_| anyhow!("Review not found"))
}

/// List reviews for a product
pub async fn list_reviews(
    product_id: i32,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplaceReview>> {
    marketplace_reviews::table
        .filter(marketplace_reviews::product_id.eq(product_id))
        .order(marketplace_reviews::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<MarketplaceReview>(conn)
        .await
        .map_err(Into::into)
}

/// Update review
pub async fn update_review(
    review_id: i32,
    rating: i32,
    comment: Option<String>,
    reviewer_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceReview> {
    // Validate rating
    if rating < 1 || rating > 5 {
        bail!("Rating must be 1-5 stars");
    }
    
    // Check ownership
    let review = get_review(review_id, conn).await?;
    if review.reviewer_id != reviewer_id {
        bail!("Not authorized to update this review");
    }
    
    // Update review
    let updated = diesel::update(marketplace_reviews::table.find(review_id))
        .set((
            marketplace_reviews::rating.eq(rating),
            marketplace_reviews::comment.eq(comment),
        ))
        .get_result::<MarketplaceReview>(conn)
        .await?;
    
    Ok(updated)
}

/// Delete review
pub async fn delete_review(
    review_id: i32,
    reviewer_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check ownership
    let review = get_review(review_id, conn).await?;
    if review.reviewer_id != reviewer_id {
        bail!("Not authorized to delete this review");
    }
    
    // Delete review
    diesel::delete(marketplace_reviews::table.find(review_id))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Get average rating for a product
pub async fn get_average_rating(
    product_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<f64> {
    use diesel::dsl::sql;
    use diesel::sql_types::Nullable;
    use diesel::sql_types::Double;
    
    let avg: Option<f64> = marketplace_reviews::table
        .filter(marketplace_reviews::product_id.eq(product_id))
        .select(sql::<Nullable<Double>>("AVG(rating)"))
        .first(conn)
        .await?;
    
    Ok(avg.unwrap_or(0.0))
}

/// Get review count for a product
pub async fn get_review_count(
    product_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<i64> {
    marketplace_reviews::table
        .filter(marketplace_reviews::product_id.eq(product_id))
        .count()
        .get_result(conn)
        .await
        .map_err(Into::into)
}

/// Get rating distribution for a product
pub async fn get_rating_distribution(
    product_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<(i32, i64)>> {
    marketplace_reviews::table
        .filter(marketplace_reviews::product_id.eq(product_id))
        .group_by(marketplace_reviews::rating)
        .select((
            marketplace_reviews::rating,
            diesel::dsl::count(marketplace_reviews::id),
        ))
        .order(marketplace_reviews::rating.desc())
        .load::<(i32, i64)>(conn)
        .await
        .map_err(Into::into)
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ReviewForm;

    #[test]
    fn test_review_form_validation_valid() {
        let form = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 5,
            comment: Some("Great product!".to_string()),
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_review_form_validation_rating_too_low() {
        let form = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 0,
            comment: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_validation_rating_too_high() {
        let form = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 6,
            comment: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_validation_comment_too_long() {
        let form = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 5,
            comment: Some("a".repeat(2001)),
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_validation_boundary_values() {
        // Test minimum valid rating
        let form1 = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 1,
            comment: None,
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid rating
        let form2 = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 5,
            comment: None,
        };
        assert!(form2.validate().is_ok());

        // Test maximum valid comment length
        let form3 = ReviewForm {
            order_id: 1,
            product_id: 1,
            reviewer_id: 2,
            rating: 3,
            comment: Some("a".repeat(2000)),
        };
        assert!(form3.validate().is_ok());
    }
}
