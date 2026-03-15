#!/bin/bash
# Script to rename ClawMeet to ClawMesh throughout the project

set -e

echo "🔄 Starting ClawMeet -> ClawMesh renaming process..."

# Function to replace in file
replace_in_file() {
    local file="$1"
    if [[ -f "$file" ]]; then
        # Use sed with backup
        sed -i.bak 's/ClawMeet/ClawMesh/g' "$file"
        sed -i.bak 's/clawmeet/clawmesh/g' "$file"
        sed -i.bak 's/CLAWMEET/CLAWMESH/g' "$file"
        rm "${file}.bak" 2>/dev/null || true
        echo "✅ Updated: $file"
    fi
}

# Export function for find -exec
export -f replace_in_file

echo "📝 Updating Rust source files..."
find . -type f -name "*.rs" -not -path "*/target/*" -not -path "*/.git/*" -exec bash -c 'replace_in_file "$0"' {} \;

echo "📝 Updating TOML files..."
find . -type f -name "*.toml" -not -path "*/target/*" -not -path "*/.git/*" -exec bash -c 'replace_in_file "$0"' {} \;

echo "📝 Updating Markdown files..."
find . -type f -name "*.md" -not -path "*/target/*" -not -path "*/.git/*" -exec bash -c 'replace_in_file "$0"' {} \;

echo "📝 Updating configuration files..."
find . -type f \( -name "*.yml" -o -name "*.yaml" -o -name "*.conf" -o -name "Dockerfile" \) -not -path "*/target/*" -not -path "*/.git/*" -exec bash -c 'replace_in_file "$0"' {} \;

echo "📝 Updating SQL files..."
find . -type f -name "*.sql" -not -path "*/target/*" -not -path "*/.git/*" -exec bash -c 'replace_in_file "$0"' {} \;

echo ""
echo "✨ Renaming complete!"
echo ""
echo "📊 Summary of changes:"
echo "   - All 'ClawMeet' -> 'ClawMesh'"
echo "   - All 'clawmeet' -> 'clawmesh'"
echo "   - All 'CLAWMEET' -> 'CLAWMESH'"
echo ""
echo "🔍 Next steps:"
echo "   1. Review changes: git diff"
echo "   2. Test compilation: cargo check --all"
echo "   3. Run tests: cargo test --all"
echo "   4. Commit changes: git add -A && git commit -m 'Rename ClawMeet to ClawMesh'"
echo ""
