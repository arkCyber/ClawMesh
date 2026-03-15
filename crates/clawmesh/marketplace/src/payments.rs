/// Payment Management Functions
/// 
/// Core functions for managing marketplace payments

use crate::models::{MarketplacePayment, PaymentForm, PaymentStatus};
use anyhow::{anyhow, bail, Result};
use diesel::prelude::*;
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_db_schema_file::schema::marketplace_payments;

/// Create a new payment
pub async fn create_payment(
    form: PaymentForm,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplacePayment> {
    // Check if order exists
    use lemmy_db_schema_file::schema::marketplace_orders;
    let order_count: i64 = marketplace_orders::table
        .filter(marketplace_orders::id.eq(form.order_id))
        .count()
        .get_result(conn)
        .await?;
    
    if order_count == 0 {
        bail!("Order not found");
    }
    
    // Check if payment already exists for this order
    let payment_count: i64 = marketplace_payments::table
        .filter(marketplace_payments::order_id.eq(form.order_id))
        .count()
        .get_result(conn)
        .await?;
    
    if payment_count > 0 {
        bail!("Payment already exists for this order");
    }
    
    // Insert payment
    let payment = diesel::insert_into(marketplace_payments::table)
        .values(&form)
        .get_result::<MarketplacePayment>(conn)
        .await?;
    
    Ok(payment)
}

/// Get payment by ID
pub async fn get_payment(
    payment_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplacePayment> {
    marketplace_payments::table
        .find(payment_id)
        .first::<MarketplacePayment>(conn)
        .await
        .map_err(|_| anyhow!("Payment not found"))
}

/// Get payment by order ID
pub async fn get_payment_by_order(
    order_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplacePayment> {
    marketplace_payments::table
        .filter(marketplace_payments::order_id.eq(order_id))
        .first::<MarketplacePayment>(conn)
        .await
        .map_err(|_| anyhow!("Payment not found"))
}

/// Process payment (simulate payment processing)
pub async fn process_payment(
    payment_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplacePayment> {
    let payment = get_payment(payment_id, conn).await?;
    
    // Check if payment is in pending status
    if payment.status != PaymentStatus::Pending as i32 {
        bail!("Payment is not in pending status");
    }
    
    // Check if payer has enough credits
    use lemmy_db_schema_file::schema::credit_history;
    let payer_balance: Option<i64> = credit_history::table
        .filter(credit_history::person_id.eq(payment.payer_id))
        .select(diesel::dsl::sum(credit_history::amount))
        .first(conn)
        .await?;
    
    let payer_balance = payer_balance.unwrap_or(0);
    
    if payer_balance < payment.amount {
        // Update payment status to failed
        let failed = diesel::update(marketplace_payments::table.find(payment_id))
            .set(marketplace_payments::status.eq(PaymentStatus::Failed as i32))
            .get_result::<MarketplacePayment>(conn)
            .await?;
        
        bail!("Insufficient credits");
    }
    
    // Deduct credits from payer
    diesel::insert_into(credit_history::table)
        .values((
            credit_history::person_id.eq(payment.payer_id),
            credit_history::amount.eq(-payment.amount),
            credit_history::description.eq(format!("Payment for order #{}", payment.order_id)),
        ))
        .execute(conn)
        .await?;
    
    // Add credits to payee
    diesel::insert_into(credit_history::table)
        .values((
            credit_history::person_id.eq(payment.payee_id),
            credit_history::amount.eq(payment.amount),
            credit_history::description.eq(format!("Payment received for order #{}", payment.order_id)),
        ))
        .execute(conn)
        .await?;
    
    // Update payment status to completed
    let completed = diesel::update(marketplace_payments::table.find(payment_id))
        .set((
            marketplace_payments::status.eq(PaymentStatus::Completed as i32),
            marketplace_payments::processed_at.eq(Some(chrono::Utc::now())),
        ))
        .get_result::<MarketplacePayment>(conn)
        .await?;
    
    Ok(completed)
}

/// Refund payment
pub async fn refund_payment(
    payment_id: i32,
    conn: &mut AsyncPgConnection,
) -> Result<MarketplacePayment> {
    let payment = get_payment(payment_id, conn).await?;
    
    // Check if payment is completed
    if payment.status != PaymentStatus::Completed as i32 {
        bail!("Payment is not completed");
    }
    
    // Refund credits to payer
    use lemmy_db_schema_file::schema::credit_history;
    diesel::insert_into(credit_history::table)
        .values((
            credit_history::person_id.eq(payment.payer_id),
            credit_history::amount.eq(payment.amount),
            credit_history::description.eq(format!("Refund for order #{}", payment.order_id)),
        ))
        .execute(conn)
        .await?;
    
    // Deduct credits from payee
    diesel::insert_into(credit_history::table)
        .values((
            credit_history::person_id.eq(payment.payee_id),
            credit_history::amount.eq(-payment.amount),
            credit_history::description.eq(format!("Refund issued for order #{}", payment.order_id)),
        ))
        .execute(conn)
        .await?;
    
    // Update payment status to refunded
    let refunded = diesel::update(marketplace_payments::table.find(payment_id))
        .set(marketplace_payments::status.eq(PaymentStatus::Refunded as i32))
        .get_result::<MarketplacePayment>(conn)
        .await?;
    
    Ok(refunded)
}

/// Get payment history for an agent
pub async fn get_payment_history(
    agent_id: i32,
    as_payer: bool,
    limit: i64,
    offset: i64,
    conn: &mut AsyncPgConnection,
) -> Result<Vec<MarketplacePayment>> {
    let query = if as_payer {
        marketplace_payments::table
            .filter(marketplace_payments::payer_id.eq(agent_id))
            .into_boxed()
    } else {
        marketplace_payments::table
            .filter(marketplace_payments::payee_id.eq(agent_id))
            .into_boxed()
    };
    
    let payments = query
        .order(marketplace_payments::created_at.desc())
        .limit(limit)
        .offset(offset)
        .load::<MarketplacePayment>(conn)
        .await?;
    
    Ok(payments)
}
