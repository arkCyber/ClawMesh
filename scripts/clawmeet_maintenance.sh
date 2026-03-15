#!/bin/bash
# ClawMeet 维护脚本

set -e

DB_URL="${DATABASE_URL:-postgres://postgres:password@localhost/lemmy}"
BACKUP_DIR="${BACKUP_DIR:-./backups}"

# 颜色输出
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() {
    echo -e "${BLUE}ℹ $1${NC}"
}

success() {
    echo -e "${GREEN}✓ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# 显示帮助
show_help() {
    cat << EOF
ClawMeet 维护工具

用法: $0 <命令>

命令:
  stats           显示统计信息
  cleanup         清理旧数据
  inactive        标记不活跃智能体
  backup          备份数据库
  restore <file>  恢复数据库
  check           健康检查

环境变量:
  DATABASE_URL    数据库连接字符串 (默认: postgres://postgres:password@localhost/lemmy)
  BACKUP_DIR      备份目录 (默认: ./backups)

示例:
  $0 stats
  $0 cleanup
  DATABASE_URL=postgres://user:pass@host/db $0 backup
EOF
}

# 显示统计信息
show_stats() {
    info "ClawMeet 统计信息"
    echo ""
    
    echo "用户统计:"
    psql "$DB_URL" -c "
        SELECT 
            user_type,
            COUNT(*) as count,
            AVG(credit_score) as avg_credit,
            MIN(credit_score) as min_credit,
            MAX(credit_score) as max_credit
        FROM person
        GROUP BY user_type;
    "
    
    echo ""
    echo "声誉等级分布:"
    psql "$DB_URL" -c "
        SELECT 
            reputation_tier,
            COUNT(*) as count
        FROM person
        WHERE user_type = 'human'
        GROUP BY reputation_tier
        ORDER BY 
            CASE reputation_tier
                WHEN 'veteran' THEN 1
                WHEN 'trusted' THEN 2
                WHEN 'regular' THEN 3
                WHEN 'newcomer' THEN 4
                ELSE 5
            END;
    "
    
    echo ""
    echo "智能体状态:"
    psql "$DB_URL" -c "
        SELECT 
            is_active,
            COUNT(*) as count,
            AVG(EXTRACT(EPOCH FROM (NOW() - last_heartbeat))/3600) as avg_hours_since_heartbeat
        FROM agent_heartbeats
        GROUP BY is_active;
    "
    
    echo ""
    echo "信用历史统计 (最近 7 天):"
    psql "$DB_URL" -c "
        SELECT 
            DATE(created_at) as date,
            COUNT(*) as actions,
            SUM(credit_change) as total_change,
            AVG(credit_change) as avg_change
        FROM credit_history
        WHERE created_at > NOW() - INTERVAL '7 days'
        GROUP BY DATE(created_at)
        ORDER BY date DESC;
    "
}

# 清理旧数据
cleanup_old_data() {
    info "清理旧数据..."
    
    # 清理 6 个月前的信用历史
    result=$(psql "$DB_URL" -t -c "
        DELETE FROM credit_history 
        WHERE created_at < NOW() - INTERVAL '6 months'
        RETURNING id;
    " | wc -l)
    
    success "删除了 $result 条旧的信用历史记录"
}

# 标记不活跃智能体
mark_inactive() {
    info "标记不活跃智能体..."
    
    result=$(psql "$DB_URL" -t -c "
        UPDATE agent_heartbeats 
        SET is_active = false 
        WHERE last_heartbeat < NOW() - INTERVAL '8 hours'
          AND is_active = true
        RETURNING person_id;
    " | wc -l)
    
    success "标记了 $result 个智能体为不活跃"
}

# 备份数据库
backup_db() {
    info "备份数据库..."
    
    mkdir -p "$BACKUP_DIR"
    backup_file="$BACKUP_DIR/clawmeet_$(date +%Y%m%d_%H%M%S).sql"
    
    pg_dump "$DB_URL" > "$backup_file"
    
    # 压缩备份
    gzip "$backup_file"
    
    success "备份完成: ${backup_file}.gz"
    
    # 显示备份大小
    size=$(du -h "${backup_file}.gz" | cut -f1)
    info "备份大小: $size"
}

# 恢复数据库
restore_db() {
    if [ -z "$1" ]; then
        echo "错误: 请指定备份文件"
        echo "用法: $0 restore <backup_file>"
        exit 1
    fi
    
    backup_file="$1"
    
    if [ ! -f "$backup_file" ]; then
        echo "错误: 文件不存在: $backup_file"
        exit 1
    fi
    
    warning "这将覆盖当前数据库！"
    read -p "确定要继续吗? (yes/no): " confirm
    
    if [ "$confirm" != "yes" ]; then
        echo "取消恢复"
        exit 0
    fi
    
    info "恢复数据库..."
    
    # 如果是 .gz 文件，先解压
    if [[ "$backup_file" == *.gz ]]; then
        gunzip -c "$backup_file" | psql "$DB_URL"
    else
        psql "$DB_URL" < "$backup_file"
    fi
    
    success "恢复完成"
}

# 健康检查
health_check() {
    info "ClawMeet 健康检查"
    echo ""
    
    # 检查数据库连接
    if psql "$DB_URL" -c "SELECT 1" > /dev/null 2>&1; then
        success "数据库连接正常"
    else
        warning "数据库连接失败"
        exit 1
    fi
    
    # 检查表是否存在
    tables=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM information_schema.tables 
        WHERE table_name IN ('person', 'credit_history', 'agent_heartbeats');
    ")
    
    if [ "$tables" -eq 3 ]; then
        success "所有必需的表都存在"
    else
        warning "缺少必需的表 (找到 $tables/3)"
    fi
    
    # 检查 person 表字段
    fields=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM information_schema.columns 
        WHERE table_name='person' 
        AND column_name IN ('user_type', 'credit_score', 'reputation_tier', 'agent_metadata');
    ")
    
    if [ "$fields" -eq 4 ]; then
        success "Person 表包含所有 ClawMeet 字段"
    else
        warning "Person 表缺少字段 (找到 $fields/4)"
    fi
    
    # 检查索引
    indexes=$(psql "$DB_URL" -t -c "
        SELECT COUNT(*) FROM pg_indexes 
        WHERE indexname LIKE 'idx_person_%' 
        AND indexname IN ('idx_person_user_type', 'idx_person_credit_score', 'idx_person_reputation_tier');
    ")
    
    if [ "$indexes" -eq 3 ]; then
        success "所有索引都存在"
    else
        warning "缺少索引 (找到 $indexes/3)"
    fi
    
    echo ""
    success "健康检查完成"
}

# 主逻辑
case "${1:-}" in
    stats)
        show_stats
        ;;
    cleanup)
        cleanup_old_data
        ;;
    inactive)
        mark_inactive
        ;;
    backup)
        backup_db
        ;;
    restore)
        restore_db "$2"
        ;;
    check)
        health_check
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo "错误: 未知命令 '${1:-}'"
        echo ""
        show_help
        exit 1
        ;;
esac
