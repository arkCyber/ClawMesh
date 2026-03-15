/// Agent Marketplace Data Models
/// 
/// Defines data structures for marketplace transactions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use lemmy_db_schema_file::schema::{marketplace_products, marketplace_orders, marketplace_payments, marketplace_reviews};

// ============================================================================
// Product Models
// ============================================================================

/// Product category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ProductCategory {
    Skill = 0,
    Service = 1,
    Data = 2,
    Tool = 3,
    Other = 4,
}

/// Product status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ProductStatus {
    Draft = 0,
    Active = 1,
    Inactive = 2,
    Sold = 3,
}

/// Marketplace product
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = marketplace_products)]
pub struct MarketplaceProduct {
    pub id: i32,
    pub seller_id: i32, // PersonId of the seller
    pub name: String,
    pub description: Option<String>,
    pub category: i32, // ProductCategory
    pub price: i64, // Price in credits (smallest unit)
    pub stock: i32,
    pub status: i32, // ProductStatus
    pub image_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Form for creating/updating product
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = marketplace_products)]
pub struct ProductForm {
    pub seller_id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: i32,
    pub price: i64,
    pub stock: i32,
    pub status: i32,
    pub image_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

// ============================================================================
// Order Models
// ============================================================================

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum OrderStatus {
    Pending = 0,
    Confirmed = 1,
    Processing = 2,
    Completed = 3,
    Cancelled = 4,
    Refunded = 5,
}

/// Marketplace order
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = marketplace_orders)]
pub struct MarketplaceOrder {
    pub id: i32,
    pub product_id: i32,
    pub buyer_id: i32, // PersonId of the buyer
    pub seller_id: i32, // PersonId of the seller
    pub quantity: i32,
    pub total_price: i64, // Total price in credits
    pub status: i32, // OrderStatus
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Form for creating order
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = marketplace_orders)]
pub struct OrderForm {
    pub product_id: i32,
    pub buyer_id: i32,
    pub seller_id: i32,
    pub quantity: i32,
    pub total_price: i64,
    pub status: i32,
    pub notes: Option<String>,
}

// ============================================================================
// Payment Models
// ============================================================================

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum PaymentStatus {
    Pending = 0,
    Processing = 1,
    Completed = 2,
    Failed = 3,
    Refunded = 4,
}

/// Marketplace payment
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = marketplace_payments)]
pub struct MarketplacePayment {
    pub id: i32,
    pub order_id: i32,
    pub payer_id: i32, // PersonId of the payer
    pub payee_id: i32, // PersonId of the payee
    pub amount: i64, // Amount in credits
    pub status: i32, // PaymentStatus
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Form for creating payment
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = marketplace_payments)]
pub struct PaymentForm {
    pub order_id: i32,
    pub payer_id: i32,
    pub payee_id: i32,
    pub amount: i64,
    pub status: i32,
    pub transaction_id: Option<String>,
}

// ============================================================================
// Review Models
// ============================================================================

/// Marketplace review
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = marketplace_reviews)]
pub struct MarketplaceReview {
    pub id: i32,
    pub product_id: i32,
    pub order_id: i32,
    pub reviewer_id: i32, // PersonId of the reviewer
    pub rating: i32, // 1-5 stars
    pub comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Form for creating/updating review
#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = marketplace_reviews)]
pub struct ReviewForm {
    pub product_id: i32,
    pub order_id: i32,
    pub reviewer_id: i32,
    pub rating: i32,
    pub comment: Option<String>,
}

// ============================================================================
// Helper Structures
// ============================================================================

/// Product with additional details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithDetails {
    pub product: MarketplaceProduct,
    pub seller_name: String,
    pub average_rating: f64,
    pub review_count: i64,
    pub total_sales: i64,
}

/// Order with additional details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderWithDetails {
    pub order: MarketplaceOrder,
    pub product_name: String,
    pub buyer_name: String,
    pub seller_name: String,
    pub payment_status: Option<i32>,
}

/// Order statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderStatistics {
    pub total_orders: i64,
    pub pending_orders: i64,
    pub completed_orders: i64,
    pub total_revenue: i64,
    pub average_order_value: f64,
}

/// Seller statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerStatistics {
    pub total_products: i64,
    pub active_products: i64,
    pub total_sales: i64,
    pub total_revenue: i64,
    pub average_rating: f64,
}

// ============================================================================
// Validation
// ============================================================================

impl ProductForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() || self.name.len() > 200 {
            anyhow::bail!("Product name must be 1-200 characters");
        }
        
        if let Some(desc) = &self.description {
            if desc.len() > 5000 {
                anyhow::bail!("Description too long (max 5000 characters)");
            }
        }
        
        if self.price < 0 {
            anyhow::bail!("Price must be non-negative");
        }
        
        if self.stock < 0 {
            anyhow::bail!("Stock must be non-negative");
        }
        
        if self.category < 0 || self.category > 4 {
            anyhow::bail!("Invalid category");
        }
        
        Ok(())
    }
}

impl OrderForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.quantity <= 0 {
            anyhow::bail!("Quantity must be positive");
        }
        
        if self.total_price <= 0 {
            anyhow::bail!("Total price must be positive");
        }
        
        if self.buyer_id == self.seller_id {
            anyhow::bail!("Buyer and seller cannot be the same");
        }
        
        Ok(())
    }
}

impl ReviewForm {
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.rating < 1 || self.rating > 5 {
            anyhow::bail!("Rating must be 1-5 stars");
        }
        
        if let Some(comment) = &self.comment {
            if comment.len() > 2000 {
                anyhow::bail!("Comment too long (max 2000 characters)");
            }
        }
        
        Ok(())
    }
}
