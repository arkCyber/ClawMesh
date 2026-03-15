/// Product Management Functions
/// 
/// Core functions for managing marketplace products

use crate::models::{MarketplaceProduct, ProductForm, ProductWithDetails, ProductStatus};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::marketplace_products;

/// Create a new product
pub async fn create_product(
    form: ProductForm,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceProduct> {
    // Validate form
    form.validate()?;
    
    // Check if seller exists and is an agent
    use lemmy_db_schema_file::schema::person;
    use diesel::dsl::count;
    let seller_count: i64 = person::table
        .filter(person::id.eq(form.seller_id))
        .filter(person::user_type.eq("agent"))
        .select(count(person::id))
        .first(conn)
        .await?;
    
    if seller_count == 0 {
        bail!("Seller not found or not an agent");
    }
    
    // Insert product
    let product = diesel::insert_into(marketplace_products::table)
        .values(&form)
        .get_result::<MarketplaceProduct>(conn)
        .await?;
    
    Ok(product)
}

/// Get product by ID
pub async fn get_product(
    product_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceProduct> {
    marketplace_products::table
        .find(product_id)
        .first::<MarketplaceProduct>(conn)
        .await
        .map_err(|_| anyhow!("Product not found"))
}

/// Get product with details
pub async fn get_product_with_details(
    product_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<ProductWithDetails> {
    let product = get_product(product_id, conn).await?;
    
    // Get seller name
    use lemmy_db_schema_file::schema::person;
    let seller_name: String = person::table
        .find(product.seller_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Get average rating
    use crate::reviews::get_average_rating;
    let average_rating = get_average_rating(product_id, conn).await.unwrap_or(0.0);
    
    // Get review count
    use lemmy_db_schema_file::schema::marketplace_reviews;
    let review_count: i64 = marketplace_reviews::table
        .filter(marketplace_reviews::product_id.eq(product_id))
        .count()
        .get_result(conn)
        .await?;
    
    // Get total sales
    use lemmy_db_schema_file::schema::marketplace_orders;
    let total_sales: i64 = marketplace_orders::table
        .filter(marketplace_orders::product_id.eq(product_id))
        .filter(marketplace_orders::status.eq(3)) // Completed
        .count()
        .get_result(conn)
        .await?;
    
    Ok(ProductWithDetails {
        product,
        seller_name,
        average_rating,
        review_count,
        total_sales,
    })
}

/// List products
pub async fn list_products(
    seller_id: Option<i32>,
    category: Option<i32>,
    status: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplaceProduct>> {
    let mut query = marketplace_products::table.into_boxed();
    
    // Filter by seller
    if let Some(sid) = seller_id {
        query = query.filter(marketplace_products::seller_id.eq(sid));
    }
    
    // Filter by category
    if let Some(cat) = category {
        query = query.filter(marketplace_products::category.eq(cat));
    }
    
    // Filter by status
    if let Some(stat) = status {
        query = query.filter(marketplace_products::status.eq(stat));
    }
    
    // Apply pagination
    let products = query
        .order(marketplace_products::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<MarketplaceProduct>(conn)
        .await?;
    
    Ok(products)
}

/// Update product
pub async fn update_product(
    product_id: i32,
    form: ProductForm,
    seller_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceProduct> {
    // Validate form
    form.validate()?;
    
    // Check ownership
    let product = get_product(product_id, conn).await?;
    if product.seller_id != seller_id {
        bail!("Not authorized to update this product");
    }
    
    // Update product
    let updated = diesel::update(marketplace_products::table.find(product_id))
        .set(&form)
        .get_result::<MarketplaceProduct>(conn)
        .await?;
    
    Ok(updated)
}

/// Delete product
pub async fn delete_product(
    product_id: i32,
    seller_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    // Check ownership
    let product = get_product(product_id, conn).await?;
    if product.seller_id != seller_id {
        bail!("Not authorized to delete this product");
    }
    
    // Check if there are pending orders
    use lemmy_db_schema_file::schema::marketplace_orders;
    let pending_orders_count: i64 = marketplace_orders::table
        .filter(marketplace_orders::product_id.eq(product_id))
        .filter(marketplace_orders::status.eq_any(vec![0, 1, 2])) // Pending, Confirmed, Processing
        .count()
        .get_result(conn)
        .await?;
    
    if pending_orders_count > 0 {
        bail!("Cannot delete product with pending orders");
    }
    
    // Delete product
    diesel::delete(marketplace_products::table.find(product_id))
        .execute(conn)
        .await?;
    
    Ok(())
}

/// Search products
pub async fn search_products(
    query: &str,
    category: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplaceProduct>> {
    let search_pattern = format!("%{}%", query);
    
    let mut db_query = marketplace_products::table
        .filter(marketplace_products::status.eq(ProductStatus::Active as i32))
        .filter(
            marketplace_products::name.ilike(&search_pattern)
                .or(marketplace_products::description.ilike(&search_pattern))
        )
        .into_boxed();
    
    // Filter by category
    if let Some(cat) = category {
        db_query = db_query.filter(marketplace_products::category.eq(cat));
    }
    
    let products = db_query
        .order(marketplace_products::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<MarketplaceProduct>(conn)
        .await?;
    
    Ok(products)
}

/// Get featured products (best sellers)
pub async fn get_featured_products(
    limit: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplaceProduct>> {
    // Get products with most completed orders
    use lemmy_db_schema_file::schema::marketplace_orders;
    
    let product_ids: Vec<i32> = marketplace_orders::table
        .filter(marketplace_orders::status.eq(3)) // Completed
        .group_by(marketplace_orders::product_id)
        .select(marketplace_orders::product_id)
        .order(diesel::dsl::count(marketplace_orders::id).desc())
        .limit(limit)
        .load(conn)
        .await?;
    
    if product_ids.is_empty() {
        return Ok(Vec::new());
    }
    
    marketplace_products::table
        .filter(marketplace_products::id.eq_any(product_ids))
        .filter(marketplace_products::status.eq(ProductStatus::Active as i32))
        .load::<MarketplaceProduct>(conn)
        .await
        .map_err(Into::into)
}

/// Update product stock
pub async fn update_stock(
    product_id: i32,
    quantity_change: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    diesel::update(marketplace_products::table.find(product_id))
        .set(marketplace_products::stock.eq(marketplace_products::stock + quantity_change))
        .execute(conn)
        .await?;
    
    Ok(())
}
