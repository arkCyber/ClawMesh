#!/bin/bash

# ClawMeet 服务器启动脚本

echo "🦀 启动 ClawMeet 服务器..."

# 检查 PostgreSQL 是否运行
if ! pgrep -x "postgres" > /dev/null; then
    echo "⚠️  PostgreSQL 未运行，请先启动 PostgreSQL"
    echo "   macOS: brew services start postgresql"
    echo "   Linux: sudo systemctl start postgresql"
    exit 1
fi

# 设置环境变量
export DATABASE_URL="postgresql://lemmy:password@localhost:5432/lemmy"
export RUST_LOG="info,clawmeet=debug"
export RUST_BACKTRACE=1

# 构建并运行服务器
echo "🔨 构建 ClawMeet 服务器..."
cargo build --release --bin lemmy_server

if [ $? -eq 0 ]; then
    echo "✅ 构建成功！"
    echo "🚀 启动服务器..."
    echo "📍 服务器将在 http://localhost:8536 启动"
    echo "📍 UI 界面: http://localhost:8536/clawmeet/"
    echo "📍 多语言 UI: http://localhost:8536/clawmeet/i18n/"
    echo "📍 API 文档: http://localhost:8536/api/v3/"
    echo ""
    echo "按 Ctrl+C 停止服务器"
    echo ""
    
    cargo run --release --bin lemmy_server
else
    echo "❌ 构建失败！"
    exit 1
fi
