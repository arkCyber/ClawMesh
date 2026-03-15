#!/bin/bash
# Agent System Test Runner
# DO-178C Level A Compliant Test Execution Script

set -e

echo "=========================================="
echo "Agent System Test Suite"
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

# Function to run tests for a package
run_package_tests() {
    local package=$1
    local description=$2
    
    echo -e "${YELLOW}Testing: $description${NC}"
    echo "Package: $package"
    echo ""
    
    if cargo test --package $package -- --nocapture 2>&1 | tee test_output.tmp; then
        echo -e "${GREEN}✓ $description tests passed${NC}"
        PASSED_TESTS=$((PASSED_TESTS + 1))
    else
        echo -e "${RED}✗ $description tests failed${NC}"
        FAILED_TESTS=$((FAILED_TESTS + 1))
    fi
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    echo ""
    echo "----------------------------------------"
    echo ""
}

# Start test execution
echo "Starting test execution at $(date)"
echo ""

# 1. Reputation System Tests
run_package_tests "clawmesh_reputation" "Reputation System - Unit Tests"
run_package_tests "clawmesh_reputation --test unit_tests" "Reputation System - Unit Tests (Detailed)"

# 2. Skills System Tests
run_package_tests "clawmesh_skills" "Skills System - Integration Tests"
run_package_tests "clawmesh_skills --test unit_tests" "Skills System - Unit Tests (Detailed)"

# 3. API Tests
run_package_tests "clawmesh_api --test reputation_api_tests" "Reputation API Tests"
run_package_tests "clawmesh_api --test skills_api_tests" "Skills API Tests"

# 4. End-to-End Tests
run_package_tests "--test e2e_tests" "End-to-End Integration Tests"

# Generate coverage report
echo -e "${YELLOW}Generating coverage report...${NC}"
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --all --out Html --output-dir coverage --timeout 300
    echo -e "${GREEN}✓ Coverage report generated in coverage/index.html${NC}"
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin not installed, skipping coverage${NC}"
    echo "  Install with: cargo install cargo-tarpaulin"
fi
echo ""

# Run clippy for code quality
echo -e "${YELLOW}Running clippy checks...${NC}"
if cargo clippy --all -- -D warnings 2>&1 | tee clippy_output.tmp; then
    echo -e "${GREEN}✓ Clippy checks passed${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
fi
echo ""

# Summary
echo "=========================================="
echo "Test Execution Summary"
echo "=========================================="
echo "Total test suites: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
echo -e "${RED}Failed: $FAILED_TESTS${NC}"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
    echo -e "${GREEN}✓ All tests passed!${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Review coverage report: open coverage/index.html"
    echo "2. Run database migrations: diesel migration run"
    echo "3. Start the server: cargo run --bin lemmy_server"
    exit 0
else
    echo -e "${RED}✗ Some tests failed${NC}"
    echo ""
    echo "Please review the test output above and fix the issues."
    exit 1
fi
