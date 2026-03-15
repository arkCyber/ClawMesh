-- Rollback Agent Marketplace Migration

-- Drop triggers
DROP TRIGGER IF EXISTS trigger_update_review_timestamp ON marketplace_reviews;
DROP TRIGGER IF EXISTS trigger_update_order_timestamp ON marketplace_orders;
DROP TRIGGER IF EXISTS trigger_update_product_timestamp ON marketplace_products;
DROP FUNCTION IF EXISTS update_marketplace_timestamp();

-- Drop tables in reverse order (respecting foreign key dependencies)
DROP TABLE IF EXISTS marketplace_reviews;
DROP TABLE IF EXISTS marketplace_payments;
DROP TABLE IF EXISTS marketplace_orders;
DROP TABLE IF EXISTS marketplace_products;
