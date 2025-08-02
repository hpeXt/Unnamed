#!/bin/bash
# CI/CD监控仪表板 - 实时显示CI状态和统计

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 检查依赖
check_dependencies() {
    if ! command -v gh &> /dev/null; then
        echo -e "${RED}❌ GitHub CLI (gh) 未安装${NC}"
        exit 1
    fi
    
    if ! command -v jq &> /dev/null; then
        echo -e "${RED}❌ jq 未安装${NC}"
        exit 1
    fi
}

# 显示标题
show_header() {
    clear
    echo -e "${BLUE}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                    CI/CD 监控仪表板                          ║${NC}"
    echo -e "${BLUE}║                  $(date '+%Y-%m-%d %H:%M:%S')                       ║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# 获取最新运行状态
get_latest_runs() {
    echo -e "${YELLOW}📊 最新运行状态${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    gh run list --limit 10 --json displayTitle,status,conclusion,workflowName,createdAt,databaseId | \
    jq -r '.[] | 
        if .conclusion == "success" then "✅"
        elif .conclusion == "failure" then "❌"
        elif .status == "in_progress" then "🔄"
        elif .status == "queued" then "⏳"
        else "❓" end + " " +
        (.createdAt | split("T")[0]) + " " +
        (.workflowName | .[0:20] | . + " " * (20 - length)) + " " +
        (.displayTitle | .[0:40])'
}

# 获取统计信息
get_statistics() {
    echo ""
    echo -e "${YELLOW}📈 过去24小时统计${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # 获取24小时内的数据
    SINCE=$(date -u -d '24 hours ago' '+%Y-%m-%dT%H:%M:%SZ')
    
    # 获取运行数据
    RUNS=$(gh api \
        -H "Accept: application/vnd.github+json" \
        -H "X-GitHub-Api-Version: 2022-11-28" \
        "/repos/{owner}/{repo}/actions/runs?created=>=$SINCE" \
        --paginate | jq -s 'add | .workflow_runs')
    
    # 计算统计
    TOTAL=$(echo "$RUNS" | jq 'length')
    SUCCESS=$(echo "$RUNS" | jq '[.[] | select(.conclusion == "success")] | length')
    FAILURE=$(echo "$RUNS" | jq '[.[] | select(.conclusion == "failure")] | length')
    IN_PROGRESS=$(echo "$RUNS" | jq '[.[] | select(.status == "in_progress")] | length')
    
    # 成功率
    if [ "$TOTAL" -gt 0 ]; then
        SUCCESS_RATE=$(( SUCCESS * 100 / TOTAL ))
    else
        SUCCESS_RATE=0
    fi
    
    # 平均运行时间
    AVG_TIME=$(echo "$RUNS" | jq -r '
        [.[] | select(.conclusion != null) | 
        (((.updated_at | fromdateiso8601) - (.created_at | fromdateiso8601)) / 60)] |
        if length > 0 then add / length else 0 end | floor')
    
    echo -e "总运行次数: ${BLUE}$TOTAL${NC}"
    echo -e "成功: ${GREEN}$SUCCESS${NC} | 失败: ${RED}$FAILURE${NC} | 进行中: ${YELLOW}$IN_PROGRESS${NC}"
    echo -e "成功率: ${GREEN}$SUCCESS_RATE%${NC}"
    echo -e "平均运行时间: ${BLUE}$AVG_TIME 分钟${NC}"
}

# 获取工作流性能
get_workflow_performance() {
    echo ""
    echo -e "${YELLOW}⚡ 工作流性能${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    gh run list --limit 50 --json workflowName,conclusion,createdAt,updatedAt | \
    jq -r '
        group_by(.workflowName) |
        map({
            workflow: .[0].workflowName,
            avg_minutes: (
                [.[] | select(.conclusion != null) | 
                (((.updatedAt | fromdateiso8601) - (.createdAt | fromdateiso8601)) / 60)] |
                if length > 0 then add / length else 0 end
            ),
            runs: length
        }) |
        sort_by(.avg_minutes) |
        reverse |
        .[:5] |
        .[] |
        "\(.workflow | .[0:30] | . + " " * (30 - length)) \(.avg_minutes | floor)分钟 (\(.runs)次)"'
}

# 获取失败原因
get_failure_analysis() {
    echo ""
    echo -e "${YELLOW}❌ 最近失败分析${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    gh run list --limit 20 --json displayTitle,conclusion,workflowName,databaseId | \
    jq -r '.[] | select(.conclusion == "failure") | 
        "\(.workflowName | .[0:20] | . + " " * (20 - length)) \(.displayTitle | .[0:35])"' | \
    head -5
}

# 实时监控模式
monitor_mode() {
    while true; do
        show_header
        get_latest_runs
        get_statistics
        get_workflow_performance
        get_failure_analysis
        
        echo ""
        echo -e "${BLUE}按 Ctrl+C 退出 | 自动刷新间隔: 30秒${NC}"
        
        sleep 30
    done
}

# 主函数
main() {
    check_dependencies
    
    if [ "$1" == "--monitor" ] || [ "$1" == "-m" ]; then
        monitor_mode
    else
        show_header
        get_latest_runs
        get_statistics
        get_workflow_performance
        get_failure_analysis
        
        echo ""
        echo -e "${BLUE}提示: 使用 --monitor 或 -m 参数启动实时监控模式${NC}"
    fi
}

# 运行主函数
main "$@"