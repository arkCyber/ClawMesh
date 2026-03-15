-- Agent Marketplace
-- Migration: Create marketplace tables for products, orders, payments, and reviews

-- ============================================================================
-- Marketplace Products Table
-- ============================================================================

CREATE TABLE marketplace_products (
    id SERIAL PRIMARY KEY,
    seller_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category INTEGER NOT NULL, -- 0=Skill, 1=Service, 2=Data, 3=Tool, 4=Other
    price BIGINT NOT NULL, -- Price in credits (smallest unit)
    stock INTEGER NOT NULL DEFAULT 0,
    status INTEGER NOT NULL DEFAULT 0, -- 0=Draft, 1=Active, 2=Inactive, 3=Sold
    image_url TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT price_positive CHECK (price >= 0),
    CONSTRAINT stock_non_negative CHECK (stock >= 0)
);

-- Indexes
CREATE INDEX idx_marketplace_products_seller ON marketplace_products(seller_id);
CREATE INDEX idx_marketplace_products_category ON marketplace_products(category);
CREATE INDEX idx_marketplace_products_status ON marketplace_products(status);
CREATE INDEX idx_marketplace_products_price ON marketplace_products(price);
CREATE INDEX idx_marketplace_products_created_at ON marketplace_products(created_at DESC);

-- ============================================================================
-- Marketplace Orders Table
-- ============================================================================

CREATE TABLE marketplace_orders (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES marketplace_products(id) ON DELETE RESTRICT,
    buyer_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    seller_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    quantity INTEGER NOT NULL,
    total_price BIGINT NOT NULL, -- Total price in credits
    status INTEGER NOT NULL DEFAULT 0, -- 0=Pending, 1=Confirmed, 2=Processing, 3=Completed, 4=Cancelled, 5=Refunded
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT quantity_positive CHECK (quantity > 0),
    CONSTRAINT total_price_positive CHECK (total_price > 0),
    CONSTRAINT no_self_purchase CHECK (buyer_id != seller_id)
);

-- Indexes
CREATE INDEX idx_marketplace_orders_product ON marketplace_orders(product_id);
CREATE INDEX idx_marketplace_orders_buyer ON marketplace_orders(buyer_id);
CREATE INDEX idx_marketplace_orders_seller ON marketplace_orders(seller_id);
CREATE INDEX idx_marketplace_orders_status ON marketplace_orders(status);
CREATE INDEX idx_marketplace_orders_created_at ON marketplace_orders(created_at DESC);

-- ============================================================================
-- Marketplace Payments Table
-- ============================================================================

CREATE TABLE marketplace_payments (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES marketplace_orders(id) ON DELETE CASCADE,
    payer_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    payee_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    amount BIGINT NOT NULL, -- Amount in credits
    status INTEGER NOT NULL DEFAULT 0, -- 0=Pending, 1=Processing, 2=Completed, 3=Failed, 4=Refunded
    transaction_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    CONSTRAINT amount_positive CHECK (amount > 0),
    UNIQUE(order_id)
);

-- Indexes
CREATE INDEX idx_marketplace_payments_order ON marketplace_payments(order_id);
CREATE INDEX idx_marketplace_payments_payer ON marketplace_payments(payer_id);
CREATE INDEX idx_marketplace_payments_payee ON marketplace_payments(payee_id);
CREATE INDEX idx_marketplace_payments_status ON marketplace_payments(status);
CREATE INDEX idx_marketplace_payments_created_at ON marketplace_payments(created_at DESC);

-- ============================================================================
-- Marketplace Reviews Table
-- ============================================================================

CREATE TABLE marketplace_reviews (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES marketplace_products(id) ON DELETE CASCADE,
    order_id INTEGER NOT NULL REFERENCES marketplace_orders(id) ON DELETE CASCADE,
    reviewer_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    rating INTEGER NOT NULL, -- 1-5 stars
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT rating_range CHECK (rating >= 1 AND rating <= 5),
    UNIQUE(order_id)
);

-- Indexes
CREATE INDEX idx_marketplace_reviews_product ON marketplace_reviews(product_id);
CREATE INDEX idx_marketplace_reviews_order ON marketplace_reviews(order_id);
CREATE INDEX idx_marketplace_reviews_reviewer ON marketplace_reviews(reviewer_id);
CREATE INDEX idx_marketplace_reviews_rating ON marketplace_reviews(rating);
CREATE INDEX idx_marketplace_reviews_created_at ON marketplace_reviews(created_at DESC);

-- ============================================================================
-- Triggers
-- ============================================================================

-- Auto-update updated_at timestamp for products
CREATE OR REPLACE FUNCTION update_marketplace_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_product_timestamp
    BEFORE UPDATE ON marketplace_products
    FOR EACH ROW
    EXECUTE FUNCTION update_marketplace_timestamp();

-- Auto-update updated_at timestamp for orders
CREATE TRIGGER trigger_update_order_timestamp
    BEFORE UPDATE ON marketplace_orders
    FOR EACH ROW
    EXECUTE FUNCTION update_marketplace_timestamp();

-- Auto-update updated_at timestamp for reviews
CREATE TRIGGER trigger_update_review_timestamp
    BEFORE UPDATE ON marketplace_reviews
    FOR EACH ROW
    EXECUTE FUNCTION update_marketplace_timestamp();

-- ============================================================================
-- Comments
-- ============================================================================

COMMENT ON TABLE marketplace_products IS 'Products and services offered by agents in the marketplace';
COMMENT ON TABLE marketplace_orders IS 'Orders placed by buyers for marketplace products';
COMMENT ON TABLE marketplace_payments IS 'Payment transactions for marketplace orders';
COMMENT ON TABLE marketplace_reviews IS 'Reviews and ratings for marketplace products';

COMMENT ON COLUMN marketplace_products.category IS '0=Skill, 1=Service, 2=Data, 3=Tool, 4=Other';
COMMENT ON COLUMN marketplace_products.status IS '0=Draft, 1=Active, 2=Inactive, 3=Sold';
COMMENT ON COLUMN marketplace_products.price IS 'Price in credits (smallest unit)';

COMMENT ON COLUMN marketplace_orders.status IS '0=Pending, 1=Confirmed, 2=Processing, 3=Completed, 4=Cancelled, 5=Refunded';
COMMENT ON COLUMN marketplace_orders.total_price IS 'Total price in credits';

COMMENT ON COLUMN marketplace_payments.status IS '0=Pending, 1=Processing, 2=Completed, 3=Failed, 4=Refunded';
COMMENT ON COLUMN marketplace_payments.amount IS 'Amount in credits';

COMMENT ON COLUMN marketplace_reviews.rating IS '1-5 stars rating';
