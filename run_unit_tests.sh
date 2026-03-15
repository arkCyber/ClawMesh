#!/bin/bash
# Unit Tests Runner
# DO-178C Level A Standard

set -e

echo "=========================================="
echo "ClawMesh Unit Tests Runner"
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

# Function to run unit tests
run_unit_tests() {
    local module_name=$1
    local package_name=$2
    
    echo -e "${BLUE}Testing: $module_name${NC}"
    
    if cargo test --package $package_name --lib 2>&1 | tee /tmp/test_${package_name}.log; then
        echo -e "${GREEN}✓ $module_name unit tests PASSED${NC}"
        PASSED=$((PASSED + 1))
    else
        echo -e "${RED}✗ $module_name unit tests FAILED${NC}"
        echo "Last 20 lines of output:"
        tail -20 /tmp/test_${package_name}.log
        FAILED=$((FAILED + 1))
    fi
    echo ""
}

echo "Running Unit Tests..."
echo "=========================================="

run_unit_tests "Reputation Module" "clawmesh_reputation"
run_unit_tests "Skills Module" "clawmesh_skills"
run_unit_tests "Workspace Module" "clawmesh_workspace"
run_unit_tests "Social Module" "clawmesh_social"
run_unit_tests "Marketplace Module" "clawmesh_marketplace"

echo "=========================================="
echo "Unit Test Summary"
echo "=========================================="
echo -e "${GREEN}Passed: $PASSED${NC}"
echo -e "${RED}Failed: $FAILED${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}✓ All unit tests passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ Some unit tests failed${NC}"
    exit 1
fi
