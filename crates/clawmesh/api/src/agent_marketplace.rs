/// Agent Marketplace API Endpoints
/// 
/// REST API handlers for marketplace features

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use clawmesh_marketplace::{
    models::*,
    products::*,
    orders::*,
    payments::*,
    reviews::*,
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub category: i32,
    pub price: i64,
    pub stock: i32,
    pub image_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<i32>,
    pub price: Option<i64>,
    pub stock: Option<i32>,
    pub status: Option<i32>,
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub product_id: i32,
    pub quantity: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderStatusRequest {
    pub status: i32,
}

#[derive(Debug, Deserialize)]
pub struct CreateReviewRequest {
    pub rating: i32,
    pub comment: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
    pub category: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct ProductQuery {
    pub seller_id: Option<i32>,
    pub category: Option<i32>,
    pub status: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct OrderQuery {
    pub buyer_id: Option<i32>,
    pub seller_id: Option<i32>,
    pub status: Option<i32>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product: ProductWithDetails,
}

#[derive(Debug, Serialize)]
pub struct ProductListResponse {
    pub products: Vec<ProductWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct OrderResponse {
    pub order: OrderWithDetails,
}

#[derive(Debug, Serialize)]
pub struct OrderListResponse {
    pub orders: Vec<OrderWithDetails>,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct ReviewListResponse {
    pub reviews: Vec<MarketplaceReview>,
    pub total: i64,
    pub average_rating: f64,
}

#[derive(Debug, Serialize)]
pub struct StatisticsResponse {
    pub statistics: OrderStatistics,
}

// ============================================================================
// Product Management Endpoints
// ============================================================================

/// POST /api/v3/marketplace/products
/// Create a new product
pub async fn create_product_handler(
    req: web::Json<CreateProductRequest>,
    seller_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let form = ProductForm {
        seller_id: *seller_id,
        name: req.name.clone(),
        description: req.description.clone(),
        category: req.category,
        price: req.price,
        stock: req.stock,
        status: 0, // Draft
        image_url: req.image_url.clone(),
        metadata: req.metadata.clone(),
    };
    
    let product = create_product(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let product_details = get_product_with_details(product.id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(ProductResponse { product: product_details }))
}

/// GET /api/v3/marketplace/products/{id}
/// Get product by ID
pub async fn get_product_handler(
    product_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let product_details = get_product_with_details(*product_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(ProductResponse { product: product_details }))
}

/// GET /api/v3/marketplace/products
/// List products
pub async fn list_products_handler(
    query: web::Query<ProductQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let products = list_products(
        query.seller_id,
        query.category,
        query.status,
        limit,
        offset,
        &mut conn,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut products_with_details = Vec::new();
    for product in products {
        if let Ok(details) = get_product_with_details(product.id, &mut conn).await {
            products_with_details.push(details);
        }
    }
    
    let total = products_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(ProductListResponse {
        products: products_with_details,
        total,
    }))
}

/// PUT /api/v3/marketplace/products/{id}
/// Update product
pub async fn update_product_handler(
    product_id: web::Path<i32>,
    req: web::Json<UpdateProductRequest>,
    seller_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let current = get_product(*product_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let form = ProductForm {
        seller_id: *seller_id,
        name: req.name.clone().unwrap_or(current.name),
        description: req.description.clone().or(current.description),
        category: req.category.unwrap_or(current.category),
        price: req.price.unwrap_or(current.price),
        stock: req.stock.unwrap_or(current.stock),
        status: req.status.unwrap_or(current.status),
        image_url: req.image_url.clone().or(current.image_url),
        metadata: current.metadata,
    };
    
    let updated = update_product(*product_id, form, *seller_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let product_details = get_product_with_details(updated.id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(ProductResponse { product: product_details }))
}

/// DELETE /api/v3/marketplace/products/{id}
/// Delete product
pub async fn delete_product_handler(
    product_id: web::Path<i32>,
    seller_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    delete_product(*product_id, *seller_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Product deleted"
    })))
}

/// GET /api/v3/marketplace/products/search
/// Search products
pub async fn search_products_handler(
    query: web::Query<SearchQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let products = search_products(&query.q, query.category, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut products_with_details = Vec::new();
    for product in products {
        if let Ok(details) = get_product_with_details(product.id, &mut conn).await {
            products_with_details.push(details);
        }
    }
    
    let total = products_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(ProductListResponse {
        products: products_with_details,
        total,
    }))
}

/// GET /api/v3/marketplace/products/featured
/// Get featured products
pub async fn get_featured_products_handler(
    query: web::Query<ProductQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(10).min(50);
    
    let products = get_featured_products(limit, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut products_with_details = Vec::new();
    for product in products {
        if let Ok(details) = get_product_with_details(product.id, &mut conn).await {
            products_with_details.push(details);
        }
    }
    
    let total = products_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(ProductListResponse {
        products: products_with_details,
        total,
    }))
}

// ============================================================================
// Order Management Endpoints
// ============================================================================

/// POST /api/v3/marketplace/orders
/// Create a new order
pub async fn create_order_handler(
    req: web::Json<CreateOrderRequest>,
    buyer_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    // Get product to determine seller and price
    let product = get_product(req.product_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let total_price = product.price * req.quantity as i64;
    
    let form = OrderForm {
        product_id: req.product_id,
        buyer_id: *buyer_id,
        seller_id: product.seller_id,
        quantity: req.quantity,
        total_price,
        status: 0, // Pending
        notes: req.notes.clone(),
    };
    
    let order = create_order(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let order_details = get_order_with_details(order.id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(OrderResponse { order: order_details }))
}

/// GET /api/v3/marketplace/orders/{id}
/// Get order by ID
pub async fn get_order_handler(
    order_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let order_details = get_order_with_details(*order_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(OrderResponse { order: order_details }))
}

/// GET /api/v3/marketplace/orders
/// List orders
pub async fn list_orders_handler(
    query: web::Query<OrderQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let orders = list_orders(
        query.buyer_id,
        query.seller_id,
        query.status,
        limit,
        offset,
        &mut conn,
    )
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let mut orders_with_details = Vec::new();
    for order in orders {
        if let Ok(details) = get_order_with_details(order.id, &mut conn).await {
            orders_with_details.push(details);
        }
    }
    
    let total = orders_with_details.len() as i64;
    
    Ok(HttpResponse::Ok().json(OrderListResponse {
        orders: orders_with_details,
        total,
    }))
}

/// PUT /api/v3/marketplace/orders/{id}/status
/// Update order status
pub async fn update_order_status_handler(
    order_id: web::Path<i32>,
    req: web::Json<UpdateOrderStatusRequest>,
    agent_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let status = match req.status {
        0 => OrderStatus::Pending,
        1 => OrderStatus::Confirmed,
        2 => OrderStatus::Processing,
        3 => OrderStatus::Completed,
        4 => OrderStatus::Cancelled,
        5 => OrderStatus::Refunded,
        _ => return Err(actix_web::error::ErrorBadRequest("Invalid status")),
    };
    
    let updated = update_order_status(*order_id, status, *agent_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    let order_details = get_order_with_details(updated.id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(OrderResponse { order: order_details }))
}

/// POST /api/v3/marketplace/orders/{id}/cancel
/// Cancel order
pub async fn cancel_order_handler(
    order_id: web::Path<i32>,
    buyer_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    cancel_order(*order_id, *buyer_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Order cancelled"
    })))
}

/// GET /api/v3/marketplace/statistics
/// Get order statistics
pub async fn get_statistics_handler(
    seller_id: Option<web::Path<i32>>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let statistics = get_order_statistics(seller_id.map(|id| *id), &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(StatisticsResponse { statistics }))
}

// ============================================================================
// Payment Endpoints
// ============================================================================

/// POST /api/v3/marketplace/orders/{id}/payment
/// Create payment for order
pub async fn create_payment_handler(
    order_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let order = get_order(*order_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let form = PaymentForm {
        order_id: *order_id,
        payer_id: order.buyer_id,
        payee_id: order.seller_id,
        amount: order.total_price,
        status: 0, // Pending
        transaction_id: None,
    };
    
    let payment = create_payment(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(payment))
}

/// POST /api/v3/marketplace/payments/{id}/process
/// Process payment
pub async fn process_payment_handler(
    payment_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let payment = process_payment(*payment_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(payment))
}

/// POST /api/v3/marketplace/payments/{id}/refund
/// Refund payment
pub async fn refund_payment_handler(
    payment_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let payment = refund_payment(*payment_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(payment))
}

// ============================================================================
// Review Endpoints
// ============================================================================

/// POST /api/v3/marketplace/orders/{id}/review
/// Create review for order
pub async fn create_review_handler(
    order_id: web::Path<i32>,
    req: web::Json<CreateReviewRequest>,
    reviewer_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let order = get_order(*order_id, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorNotFound(e.to_string()))?;
    
    let form = ReviewForm {
        product_id: order.product_id,
        order_id: *order_id,
        reviewer_id: *reviewer_id,
        rating: req.rating,
        comment: req.comment.clone(),
    };
    
    let review = create_review(form, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e.to_string()))?;
    
    Ok(HttpResponse::Ok().json(review))
}

/// GET /api/v3/marketplace/products/{id}/reviews
/// Get reviews for product
pub async fn list_reviews_handler(
    product_id: web::Path<i32>,
    query: web::Query<ProductQuery>,
    pool: web::Data<DbPool>,
) -> ActixResult<HttpResponse> {
    let mut conn = pool.get().await.map_err(|e| {
        actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))
    })?;
    
    let limit = query.limit.unwrap_or(20).min(100);
    let offset = query.offset.unwrap_or(0);
    
    let reviews = list_reviews(*product_id, limit, offset, &mut conn)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    
    let average_rating = get_average_rating(*product_id, &mut conn)
        .await
        .unwrap_or(0.0);
    
    let total = reviews.len() as i64;
    
    Ok(HttpResponse::Ok().json(ReviewListResponse {
        reviews,
        total,
        average_rating,
    }))
}

// Placeholder for DbPool type
type DbPool = deadpool::managed::Pool<diesel_async::pooled_connection::AsyncDieselConnectionManager<diesel_async::AsyncPgConnection>>;
