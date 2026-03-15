/// Agent Marketplace Module
/// 
/// Provides marketplace capabilities for agents to buy/sell products and services

pub mod models;
pub mod products;
pub mod orders;
pub mod payments;
pub mod reviews;

pub use models::{
    MarketplaceProduct,
    ProductForm,
    MarketplaceOrder,
    OrderForm,
    MarketplacePayment,
    PaymentForm,
    MarketplaceReview,
    ReviewForm,
    ProductCategory,
    ProductStatus,
    OrderStatus,
    PaymentStatus,
};

pub use products::{
    create_product,
    get_product,
    list_products,
    update_product,
    delete_product,
    search_products,
    get_featured_products,
};

pub use orders::{
    create_order,
    get_order,
    list_orders,
    update_order_status,
    cancel_order,
    get_order_statistics,
};

pub use payments::{
    create_payment,
    get_payment,
    process_payment,
    refund_payment,
    get_payment_history,
};

pub use reviews::{
    create_review,
    get_review,
    list_reviews,
    update_review,
    delete_review,
    get_average_rating,
};
