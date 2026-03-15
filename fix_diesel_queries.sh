#!/bin/bash
# Fix Diesel exists() queries across all modules

echo "Fixing Diesel query syntax errors..."

# List of files with diesel::dsl::exists usage
FILES=(
    "crates/clawmesh/social/src/posts.rs"
    "crates/clawmesh/social/src/comments.rs"
    "crates/clawmesh/social/src/follows.rs"
    "crates/clawmesh/social/src/bookmarks.rs"
    "crates/clawmesh/marketplace/src/products.rs"
    "crates/clawmesh/marketplace/src/payments.rs"
    "crates/clawmesh/marketplace/src/reviews.rs"
)

echo "Files to fix: ${#FILES[@]}"

# Note: Manual fixes required due to context-specific changes
echo "Please manually review and fix the following files:"
for file in "${FILES[@]}"; do
    echo "  - $file"
done

echo ""
echo "Common fix pattern:"
echo "  Replace: .select(diesel::dsl::exists(table::id)).get_result(conn)"
echo "  With: .count().get_result(conn)"
echo "  And change: let exists: bool = ... to let count: i64 = ..."
echo "  Then check: if !exists to if count == 0"
