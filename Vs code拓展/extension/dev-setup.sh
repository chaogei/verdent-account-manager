#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

RESTART_MODE=false

if [ "$1" = "--restart" ]; then
    RESTART_MODE=true
fi

kill_existing_processes() {
    echo "🛑 停止现有的开发进程..."
    local killed=false
    
    if pgrep -f "watch:esbuild" > /dev/null 2>&1; then
        pkill -f "watch:esbuild" && echo "  ✓ 已停止 watch:esbuild" && killed=true
    fi
    
    if pgrep -f "watch:tsc" > /dev/null 2>&1; then
        pkill -f "watch:tsc" && echo "  ✓ 已停止 watch:tsc" && killed=true
    fi
    
    if pgrep -f "vite.*webview-codeck" > /dev/null 2>&1; then
        pkill -f "vite.*webview-codeck" && echo "  ✓ 已停止 dev:webview" && killed=true
    fi
    
    if [ "$killed" = true ]; then
        echo "等待进程完全停止..."
        sleep 2
    else
        echo "  没有需要停止的进程"
    fi
    echo ""
}

check_running_processes() {
    local running=false
    if pgrep -f "watch:esbuild" > /dev/null 2>&1; then
        echo "⚠️  检测到 watch:esbuild 正在运行"
        running=true
    fi
    if pgrep -f "watch:tsc" > /dev/null 2>&1; then
        echo "⚠️  检测到 watch:tsc 正在运行"
        running=true
    fi
    if pgrep -f "vite.*webview-codeck" > /dev/null 2>&1; then
        echo "⚠️  检测到 dev:webview 正在运行"
        running=true
    fi
    
    if [ "$running" = true ]; then
        echo ""
        echo "请先停止现有的开发进程，或使用新终端窗口"
        echo "提示: 在运行脚本的终端按 Ctrl+C 可停止所有进程"
        echo "或者使用 --restart 参数自动重启: ./dev-setup.sh --restart"
        exit 1
    fi
}

echo "================================"
echo "🚀 Verdent 开发环境设置"
echo "================================"
echo ""

if [ "$RESTART_MODE" = true ]; then
    kill_existing_processes
else
    echo "🔍 检查是否有进程正在运行..."
    check_running_processes
    echo "✓ 没有冲突的进程"
    echo ""
fi

echo "📦 步骤 1/3: 安装依赖..."
npm run install:all

echo ""
echo "🔧 步骤 2/3: 生成 Protocol Buffers..."
npm run protos

echo ""
echo "🏗️  步骤 3/3: 构建 Webview..."
npm run build:webview

echo ""
echo "================================"
echo "✅ 初始设置完成！"
echo "================================"
echo ""
echo "现在启动开发监听服务..."
echo ""
echo "将在 3 个独立进程中运行："
echo "  - watch:esbuild (主扩展打包)"
echo "  - watch:tsc (类型检查)"
echo "  - dev:webview (Webview 开发服务器)"
echo ""
echo "按 Ctrl+C 停止所有进程"
echo ""

cleanup() {
    echo ""
    echo "🛑 停止所有进程..."
    jobs -p | xargs -r kill 2>/dev/null || true
    exit 0
}

trap cleanup SIGINT SIGTERM EXIT

npm run watch:esbuild &
PID1=$!
echo "✓ watch:esbuild 已启动 (PID: $PID1)"

npm run watch:tsc &
PID2=$!
echo "✓ watch:tsc 已启动 (PID: $PID2)"

cd webview-codeck
npm run dev &
PID3=$!
echo "✓ dev:webview 已启动 (PID: $PID3)"
cd ..

echo ""
echo "================================"
echo "🎉 开发环境已就绪！"
echo "================================"
echo ""
echo "所有监听服务正在运行中..."
echo "修改代码后会自动重新编译"
echo ""

wait
