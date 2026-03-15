#!/bin/bash

# ClawMeet UI 测试脚本

echo "🧪 测试 ClawMeet UI 界面..."

# 服务器地址
SERVER_URL="http://localhost:8536"

# 检查服务器是否运行
if ! curl -s "$SERVER_URL" > /dev/null 2>&1; then
    echo "❌ 服务器未运行！请先运行 ./run_clawmeet_server.sh"
    exit 1
fi

echo "✅ 服务器正在运行"
echo ""

# 测试页面列表
declare -A PAGES=(
    ["首页"]="/clawmeet/"
    ["信用系统"]="/clawmeet/credit"
    ["智能体管理"]="/clawmeet/agent"
    ["统计页面"]="/clawmeet/stats"
    ["多语言首页"]="/clawmeet/i18n/"
    ["多语言信用"]="/clawmeet/i18n/credit"
    ["多语言智能体"]="/clawmeet/i18n/agent"
    ["多语言统计"]="/clawmeet/i18n/stats"
)

# 测试所有页面
echo "📋 测试所有页面..."
for page_name in "${!PAGES[@]}"; do
    url="${PAGES[$page_name]}"
    echo -n "测试 $page_name ($url)... "
    
    if curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL$url" | grep -q "200"; then
        echo "✅ 200 OK"
    else
        echo "❌ 失败"
    fi
done

echo ""
echo "🌍 测试多语言支持..."
declare -A LANGUAGES=(
    ["中文"]="zh-CN"
    ["English"]="en"
    ["日本語"]="ja"
    ["한국어"]="ko"
    ["Français"]="fr"
    ["Deutsch"]="de"
    ["Español"]="es"
    ["Português"]="pt"
    ["Русский"]="ru"
    ["العربية"]="ar"
    ["हिन्दी"]="hi"
    ["Italiano"]="it"
    ["Nederlands"]="nl"
    ["Türkçe"]="tr"
    ["Polski"]="pl"
    ["Tiếng Việt"]="vi"
)

# 测试多语言
for lang_name in "${!LANGUAGES[@]}"; do
    lang_code="${LANGUAGES[$lang_name]}"
    url="/clawmeet/i18n/?lang=$lang_code"
    echo -n "测试 $lang_name ($lang_code)... "
    
    if curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL$url" | grep -q "200"; then
        echo "✅ 200 OK"
    else
        echo "❌ 失败"
    fi
done

echo ""
echo "🔗 测试 API 端点..."
declare -A APIS=(
    ["全局统计"]="/api/v3/credit/global/stats"
    ["智能体列表"]="/api/v3/agent/list"
    ["活跃智能体"]="/api/v3/agent/active"
)

for api_name in "${!APIS[@]}"; do
    url="${APIS[$api_name]}"
    echo -n "测试 $api_name ($url)... "
    
    if curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL$url" | grep -q "200"; then
        echo "✅ 200 OK"
    else
        echo "❌ 失败"
    fi
done

echo ""
echo "📊 生成测试报告..."
echo "测试时间: $(date)" > ui_test_report.txt
echo "服务器: $SERVER_URL" >> ui_test_report.txt
echo "" >> ui_test_report.txt
echo "页面测试结果:" >> ui_test_report.txt
for page_name in "${!PAGES[@]}"; do
    url="${PAGES[$page_name]}"
    status=$(curl -s -o /dev/null -w "%{http_code}" "$SERVER_URL$url")
    echo "$page_name ($url): $status" >> ui_test_report.txt
done

echo ""
echo "✅ UI 测试完成！"
echo "📄 详细报告: ui_test_report.txt"
echo ""
echo "🌐 在浏览器中打开以下链接测试："
echo "   首页: $SERVER_URL/clawmeet/"
echo "   多语言: $SERVER_URL/clawmeet/i18n/"
echo "   API: $SERVER_URL/api/v3/"
