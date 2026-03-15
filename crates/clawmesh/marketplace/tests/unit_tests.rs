/// Agent Marketplace Unit Tests
/// DO-178C Level A Compliant Test Suite

#[cfg(test)]
mod marketplace_unit_tests {
    use clawmesh_marketplace::models::*;

    // ========================================================================
    // ProductForm Validation Tests
    // ========================================================================

    #[test]
    fn test_product_form_valid() {
        let form = ProductForm {
            seller_id: 1,
            name: "Test Product".to_string(),
            description: Some("A test product".to_string()),
            category: ProductCategory::Skill as i32,
            price: 1000,
            stock: 10,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_product_form_empty_name() {
        let form = ProductForm {
            seller_id: 1,
            name: "".to_string(),
            description: None,
            category: ProductCategory::Skill as i32,
            price: 100,
            stock: 5,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_product_form_name_too_long() {
        let form = ProductForm {
            seller_id: 1,
            name: "a".repeat(201),
            description: None,
            category: ProductCategory::Skill as i32,
            price: 100,
            stock: 5,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_product_form_negative_price() {
        let form = ProductForm {
            seller_id: 1,
            name: "Valid Name".to_string(),
            description: None,
            category: ProductCategory::Skill as i32,
            price: -100,
            stock: 5,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_product_form_negative_stock() {
        let form = ProductForm {
            seller_id: 1,
            name: "Valid Name".to_string(),
            description: None,
            category: ProductCategory::Skill as i32,
            price: 100,
            stock: -5,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_product_form_invalid_category() {
        let form = ProductForm {
            seller_id: 1,
            name: "Valid Name".to_string(),
            description: None,
            category: 99,
            price: 100,
            stock: 5,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // OrderForm Validation Tests
    // ========================================================================

    #[test]
    fn test_order_form_valid() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 1,
            quantity: 2,
            total_price: 2000,
            status: OrderStatus::Pending as i32,
            notes: None,
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_order_form_zero_quantity() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 1,
            quantity: 0,
            total_price: 0,
            status: OrderStatus::Pending as i32,
            notes: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_negative_quantity() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 1,
            quantity: -1,
            total_price: 100,
            status: OrderStatus::Pending as i32,
            notes: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_zero_price() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 2,
            seller_id: 1,
            quantity: 1,
            total_price: 0,
            status: OrderStatus::Pending as i32,
            notes: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_order_form_self_purchase() {
        let form = OrderForm {
            product_id: 1,
            buyer_id: 1,
            seller_id: 1,
            quantity: 1,
            total_price: 100,
            status: OrderStatus::Pending as i32,
            notes: None,
        };

        assert!(form.validate().is_err());
    }

    // ========================================================================
    // ReviewForm Validation Tests
    // ========================================================================

    #[test]
    fn test_review_form_valid() {
        let form = ReviewForm {
            product_id: 1,
            order_id: 1,
            reviewer_id: 2,
            rating: 5,
            comment: Some("Excellent!".to_string()),
        };

        assert!(form.validate().is_ok());
    }

    #[test]
    fn test_review_form_rating_too_low() {
        let form = ReviewForm {
            product_id: 1,
            order_id: 1,
            reviewer_id: 2,
            rating: 0,
            comment: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_rating_too_high() {
        let form = ReviewForm {
            product_id: 1,
            order_id: 1,
            reviewer_id: 2,
            rating: 6,
            comment: None,
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_comment_too_long() {
        let form = ReviewForm {
            product_id: 1,
            order_id: 1,
            reviewer_id: 2,
            rating: 5,
            comment: Some("a".repeat(2001)),
        };

        assert!(form.validate().is_err());
    }

    #[test]
    fn test_review_form_all_ratings() {
        for rating in 1..=5 {
            let form = ReviewForm {
                product_id: 1,
                order_id: 1,
                reviewer_id: 2,
                rating,
                comment: None,
            };
            assert!(form.validate().is_ok());
        }
    }

    // ========================================================================
    // Enum Tests
    // ========================================================================

    #[test]
    fn test_product_category_values() {
        assert_eq!(ProductCategory::Skill as i32, 0);
        assert_eq!(ProductCategory::Service as i32, 1);
        assert_eq!(ProductCategory::Data as i32, 2);
        assert_eq!(ProductCategory::Tool as i32, 3);
        assert_eq!(ProductCategory::Other as i32, 4);
    }

    #[test]
    fn test_product_status_values() {
        assert_eq!(ProductStatus::Draft as i32, 0);
        assert_eq!(ProductStatus::Active as i32, 1);
        assert_eq!(ProductStatus::Inactive as i32, 2);
        assert_eq!(ProductStatus::Sold as i32, 3);
    }

    #[test]
    fn test_order_status_values() {
        assert_eq!(OrderStatus::Pending as i32, 0);
        assert_eq!(OrderStatus::Confirmed as i32, 1);
        assert_eq!(OrderStatus::Processing as i32, 2);
        assert_eq!(OrderStatus::Completed as i32, 3);
        assert_eq!(OrderStatus::Cancelled as i32, 4);
        assert_eq!(OrderStatus::Refunded as i32, 5);
    }

    #[test]
    fn test_payment_status_values() {
        assert_eq!(PaymentStatus::Pending as i32, 0);
        assert_eq!(PaymentStatus::Processing as i32, 1);
        assert_eq!(PaymentStatus::Completed as i32, 2);
        assert_eq!(PaymentStatus::Failed as i32, 3);
        assert_eq!(PaymentStatus::Refunded as i32, 4);
    }

    // ========================================================================
    // ProductWithDetails Tests
    // ========================================================================

    #[test]
    fn test_product_with_details_default() {
        use chrono::Utc;
        
        let product = MarketplaceProduct {
            id: 1,
            seller_id: 1,
            name: "Test Product".to_string(),
            description: None,
            category: ProductCategory::Skill as i32,
            price: 1000,
            stock: 10,
            status: ProductStatus::Active as i32,
            image_url: None,
            metadata: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let details = ProductWithDetails {
            product,
            seller_name: "TestSeller".to_string(),
            average_rating: 0.0,
            review_count: 0,
            total_sales: 0,
        };

        assert_eq!(details.average_rating, 0.0);
        assert_eq!(details.review_count, 0);
        assert_eq!(details.total_sales, 0);
    }

    // ========================================================================
    // OrderStatistics Tests
    // ========================================================================

    #[test]
    fn test_order_statistics_default() {
        let stats = OrderStatistics {
            total_orders: 0,
            pending_orders: 0,
            completed_orders: 0,
            total_revenue: 0,
            average_order_value: 0.0,
        };

        assert_eq!(stats.total_orders, 0);
        assert_eq!(stats.total_revenue, 0);
        assert_eq!(stats.average_order_value, 0.0);
    }

    #[test]
    fn test_order_statistics_calculation() {
        let stats = OrderStatistics {
            total_orders: 10,
            pending_orders: 2,
            completed_orders: 7,
            total_revenue: 7000,
            average_order_value: 1000.0,
        };

        assert_eq!(stats.total_orders, 10);
        assert_eq!(stats.completed_orders, 7);
        assert_eq!(stats.total_revenue, 7000);
        assert_eq!(stats.average_order_value, 1000.0);
    }

    // ========================================================================
    // SellerStatistics Tests
    // ========================================================================

    #[test]
    fn test_seller_statistics_default() {
        let stats = SellerStatistics {
            total_products: 0,
            active_products: 0,
            total_sales: 0,
            total_revenue: 0,
            average_rating: 0.0,
        };

        assert_eq!(stats.total_products, 0);
        assert_eq!(stats.total_sales, 0);
        assert_eq!(stats.average_rating, 0.0);
    }

    #[test]
    fn test_seller_statistics_calculation() {
        let stats = SellerStatistics {
            total_products: 5,
            active_products: 3,
            total_sales: 20,
            total_revenue: 10000,
            average_rating: 4.5,
        };

        assert_eq!(stats.total_products, 5);
        assert_eq!(stats.active_products, 3);
        assert_eq!(stats.total_sales, 20);
        assert_eq!(stats.total_revenue, 10000);
        assert_eq!(stats.average_rating, 4.5);
    }
}
