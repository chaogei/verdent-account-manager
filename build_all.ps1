# Verdent AI 账户管理器 - 一键打包脚本 (PowerShell)
# 编码: UTF-8

$ErrorActionPreference = "Stop"

function Write-Header {
    param([string]$Text)
    Write-Host ""
    Write-Host "======================================================================" -ForegroundColor Cyan
    Write-Host $Text -ForegroundColor Cyan
    Write-Host "======================================================================" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Success {
    param([string]$Text)
    Write-Host "[✓] $Text" -ForegroundColor Green
}

function Write-Info {
    param([string]$Text)
    Write-Host "[*] $Text" -ForegroundColor Yellow
}

function Write-Error-Custom {
    param([string]$Text)
    Write-Host "[×] $Text" -ForegroundColor Red
}

# 主流程
Write-Header "Verdent AI 账户管理器 - 一键打包工具"

Write-Info "此脚本将依次执行:"
Write-Info "  1. Python 脚本打包 (verdent_auto_register.py → exe)"
Write-Info "  2. Tauri 应用打包 (生成安装包)"
Write-Host ""
Write-Info "按任意键开始,或按 Ctrl+C 取消..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")

# 步骤 1: 检查环境
Write-Header "环境检查"

# 检查 Python
try {
    $pythonVersion = python --version 2>&1
    Write-Success "Python: $pythonVersion"
} catch {
    Write-Error-Custom "未找到 Python 环境"
    exit 1
}

# 检查 Node.js
try {
    $nodeVersion = node --version
    Write-Success "Node.js: $nodeVersion"
} catch {
    Write-Error-Custom "未找到 Node.js 环境"
    exit 1
}

# 检查 Rust
try {
    $rustVersion = rustc --version
    Write-Success "Rust: $rustVersion"
} catch {
    Write-Error-Custom "未找到 Rust 环境"
    exit 1
}

# 步骤 2: 打包 Python 脚本
Write-Header "步骤 1/2: 打包 Python 脚本"

# 检查 PyInstaller
Write-Info "检查 PyInstaller..."
try {
    pip show pyinstaller | Out-Null
    Write-Success "PyInstaller 已安装"
} catch {
    Write-Info "安装 PyInstaller..."
    pip install pyinstaller
    if (-not $?) {
        Write-Error-Custom "PyInstaller 安装失败"
        exit 1
    }
}
Write-Success "PyInstaller 已就绪"

# 清理旧文件
Write-Info "清理旧的构建文件..."
if (Test-Path "build") { Remove-Item -Recurse -Force "build" }
if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }

# 打包
Write-Info "开始打包 Python 脚本..."
pyinstaller --clean verdent_auto_register.spec

if (-not $?) {
    Write-Error-Custom "Python 打包失败"
    exit 1
}

# 检查生成的文件
if (-not (Test-Path "dist\verdent_auto_register.exe")) {
    Write-Error-Custom "未找到生成的 exe 文件"
    exit 1
}

$exeSize = (Get-Item "dist\verdent_auto_register.exe").Length / 1MB
Write-Success "打包成功! 文件大小: $([math]::Round($exeSize, 2)) MB"

# 复制到 Tauri 资源目录
Write-Info "复制到 Tauri 资源目录..."
if (-not (Test-Path "Verdent_account_manger\resources")) {
    New-Item -ItemType Directory -Path "Verdent_account_manger\resources" | Out-Null
}
Copy-Item "dist\verdent_auto_register.exe" "Verdent_account_manger\resources\verdent_auto_register.exe" -Force
Write-Success "已复制到: Verdent_account_manger\resources\verdent_auto_register.exe"

# 同时复制 Python 脚本（用于 macOS/Linux 构建）
Write-Info "复制 Python 脚本文件..."
Copy-Item "verdent_auto_register.py" "Verdent_account_manger\resources\verdent_auto_register.py" -Force
Copy-Item "verdent_auto_register_wrapper.py" "Verdent_account_manger\resources\verdent_auto_register_wrapper.py" -Force
Write-Success "已复制 Python 脚本到资源目录"

# 步骤 3: 打包 Tauri 应用
Write-Header "步骤 2/2: 打包 Tauri 应用"

Set-Location "Verdent_account_manger"

# 检查 node_modules
if (-not (Test-Path "node_modules")) {
    Write-Info "安装 npm 依赖..."
    npm install
    if (-not $?) {
        Write-Error-Custom "npm 依赖安装失败"
        Set-Location ..
        exit 1
    }
}
Write-Success "npm 依赖已就绪"

# 执行 Tauri 打包
Write-Info "开始 Tauri 打包..."
Write-Info "这可能需要几分钟时间,请耐心等待..."
npm run tauri build

if (-not $?) {
    Write-Error-Custom "Tauri 打包失败"
    Set-Location ..
    exit 1
}

Set-Location ..

# 完成
Write-Header "所有打包任务完成!"

Write-Success "生成的文件:"
Write-Info "  1. Python exe: Verdent_account_manger\resources\verdent_auto_register.exe"
Write-Info "  2. Tauri 安装包: Verdent_account_manger\src-tauri\target\release\bundle\"

Write-Host ""
Write-Info "下一步:"
Write-Info "  - 在没有 Python 环境的机器上测试安装包"
Write-Info "  - 验证自动注册功能是否正常工作"
Write-Host ""

Read-Host "按 Enter 键退出"

