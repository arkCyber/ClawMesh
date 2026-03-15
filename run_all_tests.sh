#!/bin/bash
# Complete Test Suite Runner
# DO-178C Level A Compliant Testing Script

set -e

echo "=========================================="
echo "ClawMesh Agent System - Complete Test Suite"
echo "DO-178C Level A Standard"
echo "=========================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Function to run tests
run_test_suite() {
    local suite_name=$1
    local test_command=$2
    
    echo -e "${YELLOW}Running: $suite_name${NC}"
    echo "Command: $test_command"
    
    if eval $test_command; then
        echo -e "${GREEN}✓ $suite_name PASSED${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ $suite_name FAILED${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
}

echo "Step 1: Checking compilation..."
echo "=========================================="
run_test_suite "Compilation Check" "cargo check --all"

echo "Step 2: Running Clippy (Linting)..."
echo "=========================================="
run_test_suite "Clippy Linting" "cargo clippy --all -- -D warnings"

echo "Step 3: Running Unit Tests..."
echo "=========================================="
run_test_suite "Reputation Unit Tests" "cargo test --package clawmesh_reputation --lib"
run_test_suite "Skills Unit Tests" "cargo test --package clawmesh_skills --lib"

echo "Step 4: Running Integration Tests..."
echo "=========================================="
run_test_suite "Reputation Integration Tests" "cargo test --package clawmesh_reputation --test integration_tests"
run_test_suite "Skills Integration Tests" "cargo test --package clawmesh_skills --test integration_tests"
run_test_suite "Workspace Integration Tests" "cargo test --package clawmesh_workspace --test integration_tests"
run_test_suite "Social Integration Tests" "cargo test --package clawmesh_social --test integration_tests"
run_test_suite "Marketplace Integration Tests" "cargo test --package clawmesh_marketplace --test integration_tests"

echo "Step 5: Running API Tests..."
echo "=========================================="
run_test_suite "Reputation API Tests" "cargo test --package clawmesh_api --test reputation_api_tests"
run_test_suite "Skills API Tests" "cargo test --package clawmesh_api --test skills_api_tests"
run_test_suite "Workspace API Tests" "cargo test --package clawmesh_api --test workspace_api_tests"

echo "Step 6: Running End-to-End Tests..."
echo "=========================================="
run_test_suite "E2E Tests" "cargo test --test e2e_tests"

echo "Step 7: Generating Test Coverage Report..."
echo "=========================================="
if command -v cargo-tarpaulin &> /dev/null; then
    run_test_suite "Coverage Report" "cargo tarpaulin --all --out Html --output-dir coverage"
    echo "Coverage report generated in coverage/index.html"
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin not installed, skipping coverage${NC}"
    echo "Install with: cargo install cargo-tarpaulin"
fi

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo "Total Test Suites: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    exit 1
fi
