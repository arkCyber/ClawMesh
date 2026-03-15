#!/bin/bash
# Quick Test Runner
# Tests only the newly created modules

set -e

echo "=========================================="
echo "Quick Test - New Modules Only"
echo "=========================================="
echo ""

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}Testing workspace module...${NC}"
if cargo test --package clawmesh_workspace --lib 2>&1 | tail -20; then
    echo -e "${GREEN}✓ Workspace unit tests passed${NC}"
else
    echo -e "${RED}✗ Workspace unit tests failed${NC}"
fi
echo ""

echo -e "${BLUE}Testing social module...${NC}"
if cargo test --package clawmesh_social --lib 2>&1 | tail -20; then
    echo -e "${GREEN}✓ Social unit tests passed${NC}"
else
    echo -e "${RED}✗ Social unit tests failed${NC}"
fi
echo ""

echo -e "${BLUE}Testing marketplace module...${NC}"
if cargo test --package clawmesh_marketplace --lib 2>&1 | tail -20; then
    echo -e "${GREEN}✓ Marketplace unit tests passed${NC}"
else
    echo -e "${RED}✗ Marketplace unit tests failed${NC}"
fi
echo ""

echo "=========================================="
echo "Quick test complete!"
echo "=========================================="
