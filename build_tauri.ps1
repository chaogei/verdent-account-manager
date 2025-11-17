# Verdent AI 账户管理器 - Tauri 打包工具 (PowerShell)
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

Write-Header "Verdent AI 账户管理器 - Tauri 打包工具"

# 检查 Node.js 是否安装
Write-Info "检查 Node.js 环境..."
try {
    $nodeVersion = node --version
    Write-Success "Node.js 环境检测成功: $nodeVersion"
} catch {
    Write-Error-Custom "未找到 Node.js 环境"
    Write-Info "请先安装 Node.js"
    Read-Host "按 Enter 键退出"
    exit 1
}

# 检查 Rust 是否安装
Write-Info "检查 Rust 环境..."
try {
    $rustVersion = rustc --version
    Write-Success "Rust 环境检测成功: $rustVersion"
} catch {
    Write-Error-Custom "未找到 Rust 环境"
    Write-Info "请先安装 Rust (https://rustup.rs/)"
    Read-Host "按 Enter 键退出"
    exit 1
}

# 检查 Python 打包的 exe 文件是否存在
if (-not (Test-Path "Verdent_account_manger\resources\verdent_auto_register.exe")) {
    Write-Error-Custom "未找到 Python 打包的 exe 文件"
    Write-Info "位置: Verdent_account_manger\resources\verdent_auto_register.exe"
    Write-Host ""
    $response = Read-Host "是否先运行 Python 打包? (Y/N)"
    if ($response -eq "Y" -or $response -eq "y") {
        Write-Info "运行 Python 打包脚本..."
        & .\build_python.ps1
        if (-not $?) {
            Write-Error-Custom "Python 打包失败"
            Read-Host "按 Enter 键退出"
            exit 1
        }
    } else {
        Write-Info "跳过 Python 打包,继续 Tauri 打包..."
    }
}
Write-Success "Python exe 文件已就绪"

Write-Host "开始构建 Tauri 应用..." -ForegroundColor Green

# 准备资源文件
Write-Host "`n准备资源文件..." -ForegroundColor Cyan
& "$PSScriptRoot\prepare_resources.ps1"

# 进入 Tauri 项目目录
Set-Location "Verdent_account_manger"

# 检查 node_modules 是否存在
if (-not (Test-Path "node_modules")) {
    Write-Info "安装 npm 依赖..."
    npm install
    if (-not $?) {
        Write-Error-Custom "npm 依赖安装失败"
        Set-Location ..
        Read-Host "按 Enter 键退出"
        exit 1
    }
} else {
    Write-Info "检查依赖更新..."
    npm install
}
Write-Success "npm 依赖已就绪"

# 执行 Tauri 打包
Write-Host ""
Write-Info "开始 Tauri 打包..."
Write-Info "打包步骤:"
Write-Info "  1. 前端构建 (npm run build)"
Write-Info "  2. Rust 编译 (cargo build --release)"
Write-Info "  3. 生成安装包 (MSI + NSIS)"
Write-Info "这可能需要 5-10 分钟,请耐心等待..."
Write-Host ""
npm run tauri build

if (-not $?) {
    Write-Host ""
    Write-Error-Custom "Tauri 打包失败!"
    Write-Info "常见问题:"
    Write-Info "  - 检查是否缺少图标文件"
    Write-Info "  - 检查资源文件路径是否正确"
    Write-Info "  - 查看上方的错误信息"
    Set-Location ..
    Read-Host "按 Enter 键退出"
    exit 1
}

Set-Location ..

Write-Host ""
Write-Header "Tauri 打包完成!"

Write-Host "生成的安装包位置:" -ForegroundColor Cyan
Write-Info "  - MSI 安装包: Verdent_account_manger\src-tauri\target\release\bundle\msi\"
Write-Info "  - NSIS 安装包: Verdent_account_manger\src-tauri\target\release\bundle\nsis\"
Write-Info "  - 可执行文件: Verdent_account_manger\src-tauri\target\release\verdent-account-manager.exe"
Write-Host ""

Read-Host "按 Enter 键退出"

