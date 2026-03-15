#!/bin/bash

# ClawMeet 快速启动脚本

echo "🦀 ClawMeet 快速启动"
echo "=================="

# 1. 启动 PostgreSQL
echo "1️⃣ 启动 PostgreSQL..."
brew services start postgresql@14
sleep 3

# 2. 创建数据库和用户
echo "2️⃣ 设置数据库..."
createdb lemmy 2>/dev/null || echo "   数据库已存在"
createuser -s lemmy 2>/dev/null || echo "   用户已存在"
psql -c "ALTER USER lemmy PASSWORD 'password';" 2>/dev/null || echo "   密码已设置"

# 3. 检查数据库连接
echo "3️⃣ 验证数据库连接..."
if psql -l | grep -q lemmy; then
    echo "   ✅ 数据库连接成功"
else
    echo "   ❌ 数据库连接失败"
    exit 1
fi

# 4. 设置环境变量
echo "4️⃣ 设置环境变量..."
export DATABASE_URL="postgresql://lemmy:password@localhost:5432/lemmy"
export RUST_LOG="info,clawmeet=debug"
export RUST_BACKTRACE=1

# 5. 检查端口
echo "5️⃣ 检查端口..."
if lsof -i :8536 >/dev/null 2>&1; then
    echo "   ⚠️  端口 8536 被占用，正在尝试释放..."
    lsof -ti :8536 | xargs kill -9 2>/dev/null || true
    sleep 2
fi

echo "   ✅ 端口 8536 可用"

# 6. 启动服务器
echo "6️⃣ 启动 ClawMeet 服务器..."
echo ""
echo "📍 服务器地址: http://localhost:8536"
echo "📍 UI 界面: http://localhost:8536/clawmeet/"
echo "📍 多语言: http://localhost:8536/clawmeet/i18n/"
echo ""
echo "按 Ctrl+C 停止服务器"
echo ""

# 启动服务器（忽略 lemmy_email 错误）
cargo run --bin lemmy_server 2>&1 | grep -v "lemmy_email" || cargo run --bin lemmy_server
