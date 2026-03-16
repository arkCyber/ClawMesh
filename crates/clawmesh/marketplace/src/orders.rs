/// Order Management Functions
/// 
/// Core functions for managing marketplace orders

use crate::models::{MarketplaceOrder, OrderForm, OrderWithDetails, OrderStatistics, OrderStatus};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::marketplace_orders;

/// Create a new order
pub async fn create_order(
    form: OrderForm,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceOrder> {
    // Validate form
    form.validate()?;
    
    // Check if product exists and has enough stock
    use lemmy_db_schema_file::schema::marketplace_products;
    let product: (i32, i32) = marketplace_products::table
        .find(form.product_id)
        .select((marketplace_products::stock, marketplace_products::status))
        .first(conn)
        .await?;
    
    let (stock, status) = product;
    
    if status != 1 { // Active
        bail!("Product is not available");
    }
    
    if stock < form.quantity {
        bail!("Insufficient stock");
    }
    
    // Insert order
    let order = diesel::insert_into(marketplace_orders::table)
        .values(&form)
        .get_result::<MarketplaceOrder>(conn)
        .await?;
    
    // Update product stock
    use crate::products::update_stock;
    update_stock(form.product_id, -form.quantity, conn).await?;
    
    Ok(order)
}

/// Get order by ID
pub async fn get_order(
    order_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceOrder> {
    marketplace_orders::table
        .find(order_id)
        .first::<MarketplaceOrder>(conn)
        .await
        .map_err(|_| anyhow!("Order not found"))
}

/// Get order with details
pub async fn get_order_with_details(
    order_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<OrderWithDetails> {
    let order = get_order(order_id, conn).await?;
    
    // Get product name
    use lemmy_db_schema_file::schema::marketplace_products;
    let product_name: String = marketplace_products::table
        .find(order.product_id)
        .select(marketplace_products::name)
        .first(conn)
        .await?;
    
    // Get buyer and seller names
    use lemmy_db_schema_file::schema::person;
    let buyer_name: String = person::table
        .find(order.buyer_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    let seller_name: String = person::table
        .find(order.seller_id)
        .select(person::name)
        .first(conn)
        .await?;
    
    // Get payment status
    use lemmy_db_schema_file::schema::marketplace_payments;
    let payment_status: Option<i32> = marketplace_payments::table
        .filter(marketplace_payments::order_id.eq(order_id))
        .select(marketplace_payments::status)
        .first(conn)
        .await
        .ok();
    
    Ok(OrderWithDetails {
        order,
        product_name,
        buyer_name,
        seller_name,
        payment_status,
    })
}

/// List orders
pub async fn list_orders(
    buyer_id: Option<i32>,
    seller_id: Option<i32>,
    status: Option<i32>,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplaceOrder>> {
    let mut query = marketplace_orders::table.into_boxed();
    
    // Filter by buyer
    if let Some(bid) = buyer_id {
        query = query.filter(marketplace_orders::buyer_id.eq(bid));
    }
    
    // Filter by seller
    if let Some(sid) = seller_id {
        query = query.filter(marketplace_orders::seller_id.eq(sid));
    }
    
    // Filter by status
    if let Some(stat) = status {
        query = query.filter(marketplace_orders::status.eq(stat));
    }
    
    // Apply pagination
    let orders = query
        .order(marketplace_orders::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<MarketplaceOrder>(conn)
        .await?;
    
    Ok(orders)
}

/// Update order status
pub async fn update_order_status(
    order_id: i32,
    new_status: OrderStatus,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplaceOrder> {
    // Check authorization (buyer or seller)
    let order = get_order(order_id, conn).await?;
    if order.buyer_id != agent_id && order.seller_id != agent_id {
        bail!("Not authorized to update this order");
    }
    
    // Validate status transition
    let current_status = order.status;
    if !is_valid_status_transition(current_status, new_status as i32) {
        bail!("Invalid status transition");
    }
    
    // Update order
    let mut updated = diesel::update(marketplace_orders::table.find(order_id))
        .set(marketplace_orders::status.eq(new_status as i32))
        .get_result::<MarketplaceOrder>(conn)
        .await?;
    
    // Set completed_at if status is Completed
    if new_status == OrderStatus::Completed {
        updated = diesel::update(marketplace_orders::table.find(order_id))
            .set(marketplace_orders::completed_at.eq(Some(chrono::Utc::now())))
            .get_result::<MarketplaceOrder>(conn)
            .await?;
    }
    
    Ok(updated)
}

/// Cancel order
pub async fn cancel_order(
    order_id: i32,
    agent_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<()> {
    let order = get_order(order_id, conn).await?;
    
    // Only buyer can cancel
    if order.buyer_id != agent_id {
        bail!("Only buyer can cancel order");
    }
    
    // Can only cancel pending or confirmed orders
    if order.status != OrderStatus::Pending as i32 && order.status != OrderStatus::Confirmed as i32 {
        bail!("Cannot cancel order in current status");
    }
    
    // Update status to cancelled
    diesel::update(marketplace_orders::table.find(order_id))
        .set(marketplace_orders::status.eq(OrderStatus::Cancelled as i32))
        .execute(conn)
        .await?;
    
    // Restore product stock
    use crate::products::update_stock;
    update_stock(order.product_id, order.quantity, conn).await?;
    
    Ok(())
}

/// Get order statistics
pub async fn get_order_statistics(
    seller_id: Option<i32>,
    conn: &mut AsyncPgConnection,
) -> Result<OrderStatistics> {
    // Build base filter
    let base_filter = seller_id.map(|sid| marketplace_orders::seller_id.eq(sid));
    
    // Total orders
    let mut total_query = marketplace_orders::table.into_boxed();
    if let Some(ref filter) = base_filter {
        total_query = total_query.filter(filter.clone());
    }
    let total_orders: i64 = total_query.count().get_result(conn).await?;
    
    // Pending orders
    let mut pending_query = marketplace_orders::table.into_boxed();
    if let Some(ref filter) = base_filter {
        pending_query = pending_query.filter(filter.clone());
    }
    let pending_orders: i64 = pending_query
        .filter(marketplace_orders::status.eq(OrderStatus::Pending as i32))
        .count()
        .get_result(conn)
        .await?;
    
    // Completed orders
    let mut completed_query = marketplace_orders::table.into_boxed();
    if let Some(ref filter) = base_filter {
        completed_query = completed_query.filter(filter.clone());
    }
    let completed_orders: i64 = completed_query
        .filter(marketplace_orders::status.eq(OrderStatus::Completed as i32))
        .count()
        .get_result(conn)
        .await?;
    
    // Total revenue (completed orders only)
    let mut revenue_query = marketplace_orders::table.into_boxed();
    if let Some(ref filter) = base_filter {
        revenue_query = revenue_query.filter(filter.clone());
    }
    let total_revenue: Option<i64> = revenue_query
        .filter(marketplace_orders::status.eq(OrderStatus::Completed as i32))
        .select(diesel::dsl::sum(marketplace_orders::total_price))
        .first(conn)
        .await?;
    
    let total_revenue = total_revenue.unwrap_or(0);
    
    // Average order value
    let average_order_value = if completed_orders > 0 {
        total_revenue as f64 / completed_orders as f64
    } else {
        0.0
    };
    
    Ok(OrderStatistics {
        total_orders,
        pending_orders,
        completed_orders,
        total_revenue,
        average_order_value,
    })
}

/// Validate status transition
fn is_valid_status_transition(current: i32, new: i32) -> bool {
    match (current, new) {
        // From Pending
        (0, 1) => true, // Pending -> Confirmed
        (0, 4) => true, // Pending -> Cancelled
        
        // From Confirmed
        (1, 2) => true, // Confirmed -> Processing
        (1, 4) => true, // Confirmed -> Cancelled
        
        // From Processing
        (2, 3) => true, // Processing -> Completed
        
        // From Completed
        (3, 5) => true, // Completed -> Refunded
        
        _ => false,
    }
}

// ============================================================================
// TESTS - DO-178C Level A Compliance
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OrderForm, OrderStatus};

    #[test]
    fn test_order_form_validation_valid() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: 5,
            total_price: 5000,
            shipping_address: Some("123 Test St".to_string()),
        };
        
        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_order_form_validation_zero_quantity() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: 0,
            total_price: 0,
            shipping_address: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_validation_negative_quantity() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: -1,
            total_price: 1000,
            shipping_address: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_validation_zero_price() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: 1,
            total_price: 0,
            shipping_address: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_validation_same_buyer_seller() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 2,
            quantity: 1,
            total_price: 1000,
            shipping_address: None,
        };
        
        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_status_transition_valid() {
        // Pending -> Confirmed
        assert!(is_valid_status_transition(
            OrderStatus::Pending as i32,
            OrderStatus::Confirmed as i32
        ));

        // Confirmed -> Processing
        assert!(is_valid_status_transition(
            OrderStatus::Confirmed as i32,
            OrderStatus::Processing as i32
        ));

        // Processing -> Completed
        assert!(is_valid_status_transition(
            OrderStatus::Processing as i32,
            OrderStatus::Completed as i32
        ));

        // Completed -> Refunded
        assert!(is_valid_status_transition(
            OrderStatus::Completed as i32,
            OrderStatus::Refunded as i32
        ));

        // Pending -> Cancelled
        assert!(is_valid_status_transition(
            OrderStatus::Pending as i32,
            OrderStatus::Cancelled as i32
        ));
    }

    #[test]
    fn test_order_status_transition_invalid() {
        // Completed -> Pending (invalid)
        assert!(!is_valid_status_transition(
            OrderStatus::Completed as i32,
            OrderStatus::Pending as i32
        ));

        // Cancelled -> Confirmed (invalid)
        assert!(!is_valid_status_transition(
            OrderStatus::Cancelled as i32,
            OrderStatus::Confirmed as i32
        ));

        // Processing -> Pending (invalid)
        assert!(!is_valid_status_transition(
            OrderStatus::Processing as i32,
            OrderStatus::Pending as i32
        ));
    }

    #[test]
    fn test_order_form_validation_boundary_values() {
        // Test minimum valid quantity
        let form1 = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: 1,
            total_price: 1,
            shipping_address: None,
        };
        assert!(form1.validate().is_ok());

        // Test maximum valid quantity
        let form2 = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 3,
            quantity: i32::MAX,
            total_price: i64::MAX,
            shipping_address: Some("a".repeat(500)),
        };
        assert!(form2.validate().is_ok());
    }
}
