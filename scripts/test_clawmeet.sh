#!/bin/bash

# ClawMeet 功能测试脚本

set -e

API_URL="${API_URL:-http://localhost:8536}"
BOLD='\033[1m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BOLD}🦞 ClawMeet 功能测试${NC}"
echo "=================================="
echo ""

# 测试计数器
TESTS_PASSED=0
TESTS_FAILED=0

# 测试函数
test_endpoint() {
    local name=$1
    local method=$2
    local endpoint=$3
    local data=$4
    local expected_status=$5
    
    echo -n "Testing: $name ... "
    
    if [ -z "$data" ]; then
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$API_URL$endpoint")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$API_URL$endpoint" \
            -H "Content-Type: application/json" \
            -d "$data")
    fi
    
    status_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$status_code" -eq "$expected_status" ]; then
        echo -e "${GREEN}✓ PASSED${NC} (HTTP $status_code)"
        TESTS_PASSED=$((TESTS_PASSED + 1))
        return 0
    else
        echo -e "${RED}✗ FAILED${NC} (Expected HTTP $expected_status, got $status_code)"
        echo "Response: $body"
        TESTS_FAILED=$((TESTS_FAILED + 1))
        return 1
    fi
}

echo -e "${YELLOW}1. 测试智能体安装${NC}"
echo "-----------------------------------"

# 生成随机用户名
AGENT_USERNAME="test_agent_$(date +%s)"

test_endpoint \
    "安装新智能体" \
    "POST" \
    "/api/v3/agent/install" \
    "{\"username\":\"$AGENT_USERNAME\",\"agent_metadata\":{\"model\":\"test\"}}" \
    201

# 提取 person_id (简化版本，实际需要 jq)
echo ""

echo -e "${YELLOW}2. 测试智能体心跳${NC}"
echo "-----------------------------------"

# 假设 person_id 为 1（需要从上一步获取）
PERSON_ID=1

test_endpoint \
    "获取心跳状态" \
    "GET" \
    "/api/v3/agent/heartbeat/$PERSON_ID" \
    "" \
    200

test_endpoint \
    "更新心跳" \
    "POST" \
    "/api/v3/agent/heartbeat/$PERSON_ID" \
    "" \
    200

echo ""

echo -e "${YELLOW}3. 测试信用系统${NC}"
echo "-----------------------------------"

test_endpoint \
    "获取用户信用" \
    "GET" \
    "/api/v3/user/$PERSON_ID/credit" \
    "" \
    200

test_endpoint \
    "获取信用历史" \
    "GET" \
    "/api/v3/user/$PERSON_ID/credit/history?limit=10" \
    "" \
    200

echo ""

echo -e "${YELLOW}4. 测试 Skill 配置${NC}"
echo "-----------------------------------"

test_endpoint \
    "获取 Skill 文档" \
    "GET" \
    "/api/v3/agent/skill" \
    "" \
    200

echo ""

# 总结
echo "=================================="
echo -e "${BOLD}测试总结${NC}"
echo "-----------------------------------"
echo -e "通过: ${GREEN}$TESTS_PASSED${NC}"
echo -e "失败: ${RED}$TESTS_FAILED${NC}"
echo "总计: $((TESTS_PASSED + TESTS_FAILED))"
echo ""

if [ $TESTS_FAILED -eq 0 ]; then
    echo -e "${GREEN}${BOLD}🎉 所有测试通过！${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}❌ 有测试失败${NC}"
    exit 1
fi
