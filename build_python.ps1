# Verdent AI 自动注册脚本 - Python 打包工具 (PowerShell)
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

Write-Header "Verdent AI 自动注册脚本 - Python 打包工具"

# 检查 Python 是否安装
Write-Info "检查 Python 环境..."
try {
    $pythonVersion = python --version 2>&1
    Write-Success "Python 环境检测成功: $pythonVersion"
} catch {
    Write-Error-Custom "未找到 Python 环境"
    Write-Info "请先安装 Python 3.8 或更高版本"
    Read-Host "按 Enter 键退出"
    exit 1
}

# 检查并安装 PyInstaller
Write-Info "检查 PyInstaller..."
try {
    pip show pyinstaller | Out-Null
    Write-Success "PyInstaller 已安装"
} catch {
    Write-Info "安装 PyInstaller..."
    pip install pyinstaller
    if (-not $?) {
        Write-Error-Custom "PyInstaller 安装失败"
        Read-Host "按 Enter 键退出"
        exit 1
    }
}

# 检查依赖包
Write-Info "检查 Python 依赖包..."
try {
    pip show DrissionPage | Out-Null
} catch {
    Write-Info "安装 DrissionPage..."
    pip install DrissionPage
}

try {
    pip show requests | Out-Null
} catch {
    Write-Info "安装 requests..."
    pip install requests
}
Write-Success "依赖包检查完成"

# 清理旧的构建文件
Write-Info "清理旧的构建文件..."
if (Test-Path "build") { Remove-Item -Recurse -Force "build" }
if (Test-Path "dist") { Remove-Item -Recurse -Force "dist" }
if (Test-Path "verdent_auto_register.exe") { Remove-Item -Force "verdent_auto_register.exe" }
Write-Success "清理完成"

# 使用 PyInstaller 打包
Write-Info "开始打包 Python 脚本..."
Write-Info "使用 PyInstaller 命令行参数打包..."

# 检查图标文件
$iconPath = "Verdent_account_manger\src-tauri\icons\icon.ico"
if (Test-Path $iconPath) {
    Write-Info "使用图标: $iconPath"
    pyinstaller --onefile --noconsole --icon="$iconPath" --name=verdent_auto_register --clean verdent_auto_register.py
} else {
    Write-Info "未找到图标文件，不使用图标"
    pyinstaller --onefile --noconsole --name=verdent_auto_register --clean verdent_auto_register.py
}

if (-not $?) {
    Write-Host ""
    Write-Error-Custom "打包失败!"
    Read-Host "按 Enter 键退出"
    exit 1
}

# 检查生成的 exe 文件
if (-not (Test-Path "dist\verdent_auto_register.exe")) {
    Write-Host ""
    Write-Error-Custom "未找到生成的 exe 文件"
    Read-Host "按 Enter 键退出"
    exit 1
}

Write-Host ""
Write-Success "打包成功!"
$exeSize = (Get-Item "dist\verdent_auto_register.exe").Length / 1MB
Write-Info "生成的文件: dist\verdent_auto_register.exe"
Write-Info "文件大小: $([math]::Round($exeSize, 2)) MB"

# 复制到 Tauri 资源目录
Write-Host ""
Write-Info "复制到 Tauri 资源目录..."
if (-not (Test-Path "Verdent_account_manger\resources")) {
    New-Item -ItemType Directory -Path "Verdent_account_manger\resources" | Out-Null
}
Copy-Item "dist\verdent_auto_register.exe" "Verdent_account_manger\resources\verdent_auto_register.exe" -Force

if ($?) {
    Write-Success "已复制到: Verdent_account_manger\resources\verdent_auto_register.exe"
} else {
    Write-Error-Custom "复制到 Tauri 资源目录失败"
}

Write-Host ""
Write-Header "Python 脚本打包完成!"

Write-Host "下一步:" -ForegroundColor Cyan
Write-Info "  1. 测试 exe 文件: .\dist\verdent_auto_register.exe --count 1"
Write-Info "  2. 运行 Tauri 打包: .\build_tauri.ps1"
Write-Info "  3. 或运行一键打包: .\build_all.ps1"
Write-Host ""

Read-Host "按 Enter 键退出"

