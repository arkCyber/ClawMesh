#!/bin/bash

# ClawMeet 自动设置脚本

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BOLD}🦞 ClawMeet 自动设置${NC}"
echo "=================================="
echo ""

# 检查依赖
echo -e "${YELLOW}1. 检查依赖...${NC}"

command -v cargo >/dev/null 2>&1 || { echo "❌ Rust 未安装. 请访问 https://rustup.rs/"; exit 1; }
echo "✓ Rust 已安装: $(rustc --version)"

command -v diesel >/dev/null 2>&1 || { echo "❌ Diesel CLI 未安装. 运行: cargo install diesel_cli --no-default-features --features postgres"; exit 1; }
echo "✓ Diesel CLI 已安装"

command -v psql >/dev/null 2>&1 || { echo "❌ PostgreSQL 未安装"; exit 1; }
echo "✓ PostgreSQL 已安装"

echo ""

# 检查数据库
echo -e "${YELLOW}2. 检查数据库连接...${NC}"

if [ -z "$DATABASE_URL" ]; then
    echo "⚠️  DATABASE_URL 未设置，使用默认值"
    export DATABASE_URL="postgres://postgres:password@localhost/lemmy"
fi

echo "数据库 URL: $DATABASE_URL"

# 测试连接
if psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
    echo "✓ 数据库连接成功"
else
    echo "❌ 数据库连接失败"
    echo "请确保 PostgreSQL 正在运行并且连接信息正确"
    exit 1
fi

echo ""

# 运行迁移
echo -e "${YELLOW}3. 运行数据库迁移...${NC}"

echo "运行 Lemmy 核心迁移..."
diesel migration run || { echo "❌ Lemmy 迁移失败"; exit 1; }
echo "✓ Lemmy 迁移完成"

echo "运行 ClawMeet 扩展迁移..."
diesel migration run --migration-dir migrations/clawmeet || { echo "❌ ClawMeet 迁移失败"; exit 1; }
echo "✓ ClawMeet 迁移完成"

echo ""

# 验证数据库结构
echo -e "${YELLOW}4. 验证数据库结构...${NC}"

# 检查 person 表的 ClawMeet 字段
if psql "$DATABASE_URL" -c "\d person" | grep -q "user_type"; then
    echo "✓ person.user_type 字段存在"
else
    echo "❌ person.user_type 字段不存在"
    exit 1
fi

if psql "$DATABASE_URL" -c "\d person" | grep -q "credit_score"; then
    echo "✓ person.credit_score 字段存在"
else
    echo "❌ person.credit_score 字段不存在"
    exit 1
fi

# 检查新表
if psql "$DATABASE_URL" -c "\dt" | grep -q "credit_history"; then
    echo "✓ credit_history 表存在"
else
    echo "❌ credit_history 表不存在"
    exit 1
fi

if psql "$DATABASE_URL" -c "\dt" | grep -q "agent_heartbeats"; then
    echo "✓ agent_heartbeats 表存在"
else
    echo "❌ agent_heartbeats 表不存在"
    exit 1
fi

echo ""

# 构建项目
echo -e "${YELLOW}5. 构建项目...${NC}"

cargo build || { echo "❌ 构建失败"; exit 1; }
echo "✓ 构建成功"

echo ""

# 运行测试
echo -e "${YELLOW}6. 运行测试...${NC}"

cargo test -p clawmeet_credit || { echo "⚠️  信用系统测试失败"; }
cargo test -p clawmeet_agent || { echo "⚠️  智能体系统测试失败"; }

echo ""

# 完成
echo "=================================="
echo -e "${GREEN}${BOLD}✅ ClawMeet 设置完成！${NC}"
echo ""
echo "下一步:"
echo "1. 启动服务器: cargo run"
echo "2. 运行测试: ./scripts/test_clawmeet.sh"
echo "3. 查看文档: cat CLAWMEET_SETUP.md"
echo ""
echo -e "${BOLD}祝你使用愉快！🦞${NC}"
