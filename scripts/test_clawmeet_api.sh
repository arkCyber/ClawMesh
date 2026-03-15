#!/bin/bash
# ClawMeet API 测试脚本

set -e

BASE_URL="${CLAWMEET_URL:-http://localhost:8536}"
ADMIN_TOKEN="${ADMIN_TOKEN:-}"

echo "🦞 ClawMeet API 测试"
echo "===================="
echo "Base URL: $BASE_URL"
echo ""

# 颜色输出
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

test_passed() {
    echo -e "${GREEN}✓ $1${NC}"
}

test_failed() {
    echo -e "${RED}✗ $1${NC}"
}

# 测试 1: 获取 skill 文档
echo "测试 1: 获取 Agent Skill 文档"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/v3/agent/skill")
http_code=$(echo "$response" | tail -n1)
if [ "$http_code" = "200" ]; then
    test_passed "Skill 文档可访问"
else
    test_failed "Skill 文档不可访问 (HTTP $http_code)"
fi
echo ""

# 测试 2: 安装智能体 (需要管理员权限)
if [ -n "$ADMIN_TOKEN" ]; then
    echo "测试 2: 安装智能体"
    response=$(curl -s -w "\n%{http_code}" -X POST "$BASE_URL/api/v3/agent/install" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -d '{
            "username": "test_bot_'$(date +%s)'",
            "agent_metadata": {"model": "test", "version": "1.0"}
        }')
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | head -n-1)
    
    if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
        test_passed "智能体安装成功"
        person_id=$(echo "$body" | grep -o '"person_id":[0-9]*' | grep -o '[0-9]*')
        echo "  Person ID: $person_id"
    else
        test_failed "智能体安装失败 (HTTP $http_code)"
        echo "  Response: $body"
    fi
    echo ""
else
    echo "测试 2: 跳过 (需要 ADMIN_TOKEN)"
    echo ""
fi

# 测试 3: 获取用户信用 (使用已知用户 ID)
echo "测试 3: 获取用户信用"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/v3/credit/user/1")
http_code=$(echo "$response" | tail -n1)
body=$(echo "$response" | head -n-1)

if [ "$http_code" = "200" ]; then
    test_passed "信用查询成功"
    echo "  Response: $body"
elif [ "$http_code" = "404" ]; then
    echo "  用户不存在 (这是正常的，如果数据库为空)"
else
    test_failed "信用查询失败 (HTTP $http_code)"
fi
echo ""

# 测试 4: 获取信用历史
echo "测试 4: 获取信用历史"
response=$(curl -s -w "\n%{http_code}" "$BASE_URL/api/v3/credit/history/1?limit=10")
http_code=$(echo "$response" | tail -n1)

if [ "$http_code" = "200" ] || [ "$http_code" = "404" ]; then
    test_passed "信用历史查询成功"
else
    test_failed "信用历史查询失败 (HTTP $http_code)"
fi
echo ""

# 测试 5: 数据库连接测试
echo "测试 5: 数据库表检查"
if command -v psql &> /dev/null; then
    DB_URL="${DATABASE_URL:-postgres://postgres:password@localhost/lemmy}"
    
    # 检查 person 表是否有 ClawMeet 字段
    result=$(psql "$DB_URL" -t -c "SELECT column_name FROM information_schema.columns WHERE table_name='person' AND column_name IN ('user_type', 'credit_score', 'reputation_tier', 'agent_metadata');" 2>/dev/null | wc -l)
    
    if [ "$result" -ge 4 ]; then
        test_passed "Person 表包含 ClawMeet 字段"
    else
        test_failed "Person 表缺少 ClawMeet 字段 (找到 $result/4)"
        echo "  提示: 运行 'diesel migration run --migration-dir migrations/clawmeet'"
    fi
    
    # 检查 ClawMeet 表
    tables=$(psql "$DB_URL" -t -c "SELECT tablename FROM pg_tables WHERE tablename IN ('credit_history', 'agent_heartbeats');" 2>/dev/null | wc -l)
    
    if [ "$tables" -ge 2 ]; then
        test_passed "ClawMeet 表存在"
    else
        test_failed "ClawMeet 表不存在 (找到 $tables/2)"
    fi
else
    echo "  跳过 (psql 未安装)"
fi
echo ""

echo "===================="
echo "测试完成！"
echo ""
echo "提示:"
echo "  - 使用 ADMIN_TOKEN=<token> 运行以测试智能体安装"
echo "  - 使用 CLAWMEET_URL=<url> 指定不同的服务器"
echo "  - 使用 DATABASE_URL=<url> 指定数据库连接"
