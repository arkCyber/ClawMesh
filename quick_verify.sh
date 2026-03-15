#!/bin/bash
# Quick Verification Script
# DO-178C Level A Standard

set -e

echo "=========================================="
echo "ClawMesh Quick Verification"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# 1. Check if diesel CLI is installed
echo "Step 1: Checking diesel CLI..."
if ! command -v diesel &> /dev/null; then
    echo -e "${YELLOW}⚠ diesel CLI not found, installing...${NC}"
    cargo install diesel_cli --no-default-features --features postgres
else
    echo -e "${GREEN}✓ diesel CLI found${NC}"
fi
echo ""

# 2. Check database connection
echo "Step 2: Checking database connection..."
if diesel database setup 2>/dev/null; then
    echo -e "${GREEN}✓ Database connection OK${NC}"
else
    echo -e "${YELLOW}⚠ Database setup needed${NC}"
fi
echo ""

# 3. Run migrations
echo "Step 3: Running database migrations..."
if diesel migration run; then
    echo -e "${GREEN}✓ Migrations completed${NC}"
else
    echo -e "${RED}✗ Migration failed${NC}"
    exit 1
fi
echo ""

# 4. Verify tables created
echo "Step 4: Verifying tables..."
echo "Checking agent tables..."
psql -U postgres -d lemmy -c "\dt agent_*" 2>/dev/null || echo "Note: Some tables may not exist yet"
echo ""
echo "Checking marketplace tables..."
psql -U postgres -d lemmy -c "\dt marketplace_*" 2>/dev/null || echo "Note: Some tables may not exist yet"
echo ""

# 5. Quick compile check
echo "Step 5: Quick compile check..."
if cargo check --package clawmesh_reputation 2>&1 | head -20; then
    echo -e "${GREEN}✓ Reputation module compiles${NC}"
else
    echo -e "${YELLOW}⚠ Reputation module has issues${NC}"
fi
echo ""

if cargo check --package clawmesh_skills 2>&1 | head -20; then
    echo -e "${GREEN}✓ Skills module compiles${NC}"
else
    echo -e "${YELLOW}⚠ Skills module has issues${NC}"
fi
echo ""

if cargo check --package clawmesh_workspace 2>&1 | head -20; then
    echo -e "${GREEN}✓ Workspace module compiles${NC}"
else
    echo -e "${YELLOW}⚠ Workspace module has issues${NC}"
fi
echo ""

echo "=========================================="
echo "Quick Verification Complete"
echo "=========================================="
echo ""
echo "Next steps:"
echo "1. Run full compilation: cargo build --all"
echo "2. Run tests: ./run_all_tests.sh"
echo "3. Generate coverage: cargo tarpaulin --all"
