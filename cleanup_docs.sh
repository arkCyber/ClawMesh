#!/bin/bash
# Clean up duplicate and temporary documentation files
# Keep only essential final documents

set -e

echo "🧹 Starting documentation cleanup..."

# Create backup directory
mkdir -p backup_docs
echo "📦 Backing up original documents..."
cp *.md backup_docs/ 2>/dev/null || true

echo "📊 Current document count: $(ls -1 *.md 2>/dev/null | wc -l)"

# Delete test report documents
echo "🗑️ Removing test report documents..."
rm -f \
  CLAWMEET_TEST_REPORT.md \
  PERFORMANCE_TESTING_PLAN.md \
  CLAWMEET_TESTING_COMPLETE.md \
  FINAL_CODE_COMPLETION_AND_TEST_REPORT.md \
  CLAWMEET_COMPLETE_AUDIT_AND_TEST_REPORT.md \
  INTEGRATION_TESTING_PLAN.md \
  CLAWMEET_FINAL_TEST_REPORT.md

# Delete duplicate audit reports
echo "🗑️ Removing duplicate audit reports..."
rm -f \
  AEROSPACE_GRADE_CODE_COMPLETION_REPORT.md \
  AEROSPACE_GRADE_COMPLETION_REPORT.md \
  CLAWMEET_AUDIT_REPORT.md \
  CLAWMEET_CODE_AUDIT_REPORT.md \
  CLAWMEET_CODE_COMPLETION_REPORT.md \
  CLAWMEET_COMPLETE_AUDIT_AND_TEST_REPORT.md \
  CLAWMEET_COMPLETION_REPORT.md \
  CLAWMEET_COMPREHENSIVE_AUDIT_REPORT.md \
  CLAWMEET_DEEP_AUDIT_REPORT.md \
  CLAWMEET_FINAL_COMPLETE_REPORT.md \
  CLAWMEET_FINAL_REPORT.md \
  CLAWMEET_FINAL_TEST_REPORT.md \
  CODE_AUDIT_REPORT.md \
  CODE_CLEANUP_COMPLETE_REPORT.md \
  FINAL_CODE_COMPLETION_AND_TEST_REPORT.md \
  FINAL_COMPLETION_REPORT.md \
  FINAL_INTEGRATION_REPORT.md \
  P0_COMPLETION_REPORT.md \
  P0_FINAL_COMPLETION_REPORT.md \
  P2P_CODE_AUDIT_REPORT.md \
  PHASE2_CODE_COMPLETION_REPORT.md \
  PHASE_2_3_COMPLETION_REPORT.md \
  PLACEHOLDER_CODE_AUDIT_REPORT.md \
  100K_USERS_CAPACITY_AUDIT.md \
  AEROSPACE_GRADE_AUDIT_CHECKLIST.md \
  CLAWMEET_AUDIT_COMPLETE.md \
  CLAWMEET_COMPLETE_AUDIT.md \
  CLAWMEET_COMPREHENSIVE_AUDIT_REPORT.md

# Delete other temporary documents
echo "🗑️ Removing other temporary documents..."
rm -f \
  CODE_INTEGRATION_AUDIT.md \
  FINAL_AUDIT_AND_FIXES_SUMMARY.md \
  LEMMY_FEATURE_UTILIZATION_AUDIT.md \
  SECURITY_AUDIT_PLAN.md \
  TONIGHT_CODE_AUDIT_SUMMARY.md

echo "✨ Cleanup completed!"
echo ""
echo "📊 Final document count: $(ls -1 *.md 2>/dev/null | wc -l)"
echo ""
echo "📋 Remaining essential documents:"
ls -1 *.md | sort
echo ""
echo "📦 Backup created in: backup_docs/"
echo ""
echo "🔍 Next steps:"
echo "   1. Review remaining documents"
echo "   2. Run 'git status' to see changes"
echo "   3. Commit: git add -A && git commit -m 'Clean up documentation'"
echo ""
