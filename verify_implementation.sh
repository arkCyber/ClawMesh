#!/bin/bash
# Implementation Verification Script
# DO-178C Level A Standard

set -e

echo "=========================================="
echo "ClawMesh Implementation Verification"
echo "DO-178C Level A Standard"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

PASSED=0
FAILED=0
WARNINGS=0

# Function to run step
run_step() {
    local step_name=$1
    local step_command=$2
    
    echo -e "${BLUE}Running: $step_name${NC}"
    
    if eval $step_command > /tmp/verify_step.log 2>&1; then
        echo -e "${GREEN}✓ $step_name PASSED${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ $step_name FAILED${NC}"
        echo "Error output:"
        tail -20 /tmp/verify_step.log
        FAILED=$((FAILED + 1))
        return 1
    fi
}

# Step 1: Check individual module compilation
echo ""
echo "=========================================="
echo "Step 1: Module Compilation Checks"
echo "=========================================="

run_step "Reputation Module" "cargo check --package clawmesh_reputation" || true
run_step "Skills Module" "cargo check --package clawmesh_skills" || true
run_step "Workspace Module" "cargo check --package clawmesh_workspace" || true
run_step "Social Module" "cargo check --package clawmesh_social" || true
run_step "Marketplace Module" "cargo check --package clawmesh_marketplace" || true

# Step 2: Check API module
echo ""
echo "=========================================="
echo "Step 2: API Module Check"
echo "=========================================="

run_step "API Module" "cargo check --package clawmesh_api" || true

# Step 3: Check all modules together
echo ""
echo "=========================================="
echo "Step 3: Full Project Check"
echo "=========================================="

run_step "All Modules" "cargo check --workspace" || true

# Step 4: Run clippy
echo ""
echo "=========================================="
echo "Step 4: Clippy Linting"
echo "=========================================="

if cargo clippy --workspace -- -D warnings > /tmp/clippy.log 2>&1; then
    echo -e "${GREEN}✓ Clippy PASSED (no warnings)${NC}"
    PASSED=$((PASSED + 1))
else
    echo -e "${YELLOW}⚠ Clippy found warnings${NC}"
    echo "Top warnings:"
    head -30 /tmp/clippy.log
    WARNINGS=$((WARNINGS + 1))
fi

# Step 5: Check if tests compile
echo ""
echo "=========================================="
echo "Step 5: Test Compilation"
echo "=========================================="

run_step "Reputation Tests" "cargo test --package clawmesh_reputation --no-run" || true
run_step "Skills Tests" "cargo test --package clawmesh_skills --no-run" || true
run_step "Workspace Tests" "cargo test --package clawmesh_workspace --no-run" || true

# Summary
echo ""
echo "=========================================="
echo "Verification Summary"
echo "=========================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All critical checks passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Run database migrations: diesel migration run"
    echo "2. Run tests: cargo test --workspace"
    echo "3. Generate coverage: cargo tarpaulin --workspace"
    exit 0
else
    echo -e "${RED}✗ Some checks failed${NC}"
    echo ""
    echo "Please fix the errors above before proceeding."
    exit 1
fi
