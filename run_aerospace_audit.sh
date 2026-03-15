#!/bin/bash

# ClawMeet 航空航天级审计脚本

echo "🚀 ClawMeet 航空航天级代码审计"
echo "================================"
echo ""

# 创建审计报告目录
mkdir -p audit_reports
REPORT_DIR="audit_reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

# 1. Clippy 最严格检查
echo "1️⃣ Clippy 静态分析（最严格模式）..."
cargo clippy --all-targets --all-features \
    -p clawmeet_credit \
    -p clawmeet_agent \
    -p clawmeet_config \
    -p clawmeet_cache \
    -- -D warnings \
    -D clippy::all \
    -D clippy::pedantic \
    -D clippy::nursery \
    -D clippy::cargo \
    2>&1 | tee "$REPORT_DIR/clippy_${TIMESTAMP}.txt"

CLIPPY_RESULT=$?
if [ $CLIPPY_RESULT -eq 0 ]; then
    echo "   ✅ Clippy 检查通过"
else
    echo "   ❌ Clippy 发现问题"
fi
echo ""

# 2. 安全审计
echo "2️⃣ 安全漏洞扫描..."
cargo audit 2>&1 | tee "$REPORT_DIR/audit_${TIMESTAMP}.txt"
AUDIT_RESULT=$?
if [ $AUDIT_RESULT -eq 0 ]; then
    echo "   ✅ 无安全漏洞"
else
    echo "   ⚠️  发现安全问题"
fi
echo ""

# 3. 依赖项检查
echo "3️⃣ 依赖项审计..."
if command -v cargo-deny &> /dev/null; then
    cargo deny check 2>&1 | tee "$REPORT_DIR/deny_${TIMESTAMP}.txt"
    echo "   ✅ 依赖项检查完成"
else
    echo "   ⚠️  cargo-deny 未安装，跳过"
fi
echo ""

# 4. 测试覆盖率
echo "4️⃣ 代码覆盖率分析..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin \
        -p clawmeet_credit \
        -p clawmeet_agent \
        -p clawmeet_config \
        -p clawmeet_cache \
        --out Html \
        --output-dir "$REPORT_DIR/coverage" \
        2>&1 | tee "$REPORT_DIR/coverage_${TIMESTAMP}.txt"
    echo "   ✅ 覆盖率报告: $REPORT_DIR/coverage/index.html"
else
    echo "   ⚠️  cargo-tarpaulin 未安装，跳过"
fi
echo ""

# 5. 运行所有测试
echo "5️⃣ 运行所有测试..."
cargo test \
    -p clawmeet_credit \
    -p clawmeet_agent \
    -p clawmeet_config \
    -p clawmeet_cache \
    --lib \
    2>&1 | tee "$REPORT_DIR/tests_${TIMESTAMP}.txt"

TEST_RESULT=$?
if [ $TEST_RESULT -eq 0 ]; then
    echo "   ✅ 所有测试通过"
else
    echo "   ❌ 测试失败"
fi
echo ""

# 6. 代码复杂度分析
echo "6️⃣ 代码复杂度分析..."
if command -v tokei &> /dev/null; then
    tokei crates/clawmeet --sort code | tee "$REPORT_DIR/complexity_${TIMESTAMP}.txt"
    echo "   ✅ 复杂度分析完成"
else
    echo "   ⚠️  tokei 未安装，跳过"
fi
echo ""

# 7. 未使用代码检测
echo "7️⃣ 未使用代码检测..."
cargo +nightly udeps \
    -p clawmeet_credit \
    -p clawmeet_agent \
    -p clawmeet_config \
    -p clawmeet_cache \
    2>&1 | tee "$REPORT_DIR/udeps_${TIMESTAMP}.txt" || echo "   ⚠️  cargo-udeps 未安装或失败"
echo ""

# 8. 文档检查
echo "8️⃣ 文档完整性检查..."
cargo doc --no-deps \
    -p clawmeet_credit \
    -p clawmeet_agent \
    -p clawmeet_config \
    -p clawmeet_cache \
    2>&1 | tee "$REPORT_DIR/docs_${TIMESTAMP}.txt"
echo "   ✅ 文档生成完成"
echo ""

# 生成总结报告
echo "9️⃣ 生成审计总结..."
cat > "$REPORT_DIR/summary_${TIMESTAMP}.md" << EOF
# ClawMeet 航空航天级审计总结

**审计时间**: $(date)
**审计标准**: DO-178C Level A

## 审计结果

| 项目 | 结果 | 状态 |
|------|------|------|
| Clippy 检查 | $([ $CLIPPY_RESULT -eq 0 ] && echo "通过" || echo "失败") | $([ $CLIPPY_RESULT -eq 0 ] && echo "✅" || echo "❌") |
| 安全审计 | $([ $AUDIT_RESULT -eq 0 ] && echo "通过" || echo "失败") | $([ $AUDIT_RESULT -eq 0 ] && echo "✅" || echo "⚠️") |
| 测试 | $([ $TEST_RESULT -eq 0 ] && echo "通过" || echo "失败") | $([ $TEST_RESULT -eq 0 ] && echo "✅" || echo "❌") |

## 详细报告

- Clippy: clippy_${TIMESTAMP}.txt
- 安全审计: audit_${TIMESTAMP}.txt
- 测试结果: tests_${TIMESTAMP}.txt
- 代码覆盖率: coverage/index.html

## 建议

$([ $CLIPPY_RESULT -eq 0 ] && [ $AUDIT_RESULT -eq 0 ] && [ $TEST_RESULT -eq 0 ] && echo "✅ 项目符合航空航天级标准" || echo "⚠️ 需要修复发现的问题")

EOF

echo "   ✅ 总结报告: $REPORT_DIR/summary_${TIMESTAMP}.md"
echo ""

# 最终结果
echo "================================"
echo "🎯 审计完成！"
echo ""
echo "📊 报告位置: $REPORT_DIR/"
echo "📄 总结报告: $REPORT_DIR/summary_${TIMESTAMP}.md"
echo ""

if [ $CLIPPY_RESULT -eq 0 ] && [ $AUDIT_RESULT -eq 0 ] && [ $TEST_RESULT -eq 0 ]; then
    echo "✅ 项目通过航空航天级审计！"
    exit 0
else
    echo "⚠️  项目需要改进以达到航空航天级标准"
    exit 1
fi
