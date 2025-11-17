# 准备资源文件脚本
# 确保 resources 目录包含所需的文件

Write-Host "准备资源文件..." -ForegroundColor Green

# 创建 resources 目录
$resourcesDir = "Verdent_account_manger\resources"
if (!(Test-Path $resourcesDir)) {
    New-Item -ItemType Directory -Path $resourcesDir -Force | Out-Null
    Write-Host "创建资源目录: $resourcesDir" -ForegroundColor Yellow
}

# 复制 Python 脚本到 resources 目录
$scriptsTooCopy = @(
    @{Source = "verdent_auto_register.py"; Dest = "$resourcesDir\verdent_auto_register.py"},
    @{Source = "verdent_auto_register_wrapper.py"; Dest = "$resourcesDir\verdent_auto_register_wrapper.py"}
)

foreach ($script in $scriptsTooCopy) {
    if (Test-Path $script.Source) {
        Copy-Item -Path $script.Source -Destination $script.Dest -Force
        Write-Host "复制文件: $($script.Source) -> $($script.Dest)" -ForegroundColor Cyan
    } else {
        Write-Host "警告: 文件不存在 - $($script.Source)" -ForegroundColor Yellow
    }
}

# 如果存在 Python 可执行文件，也复制它
$exePath = "dist\verdent_auto_register.exe"
if (Test-Path $exePath) {
    Copy-Item -Path $exePath -Destination "$resourcesDir\verdent_auto_register.exe" -Force
    Write-Host "复制可执行文件: $exePath -> $resourcesDir\verdent_auto_register.exe" -ForegroundColor Cyan
} else {
    Write-Host "提示: Python 可执行文件不存在 (仅 Windows 需要)" -ForegroundColor Gray
}

Write-Host "`n资源准备完成!" -ForegroundColor Green
Write-Host "现在可以运行 'npm run tauri dev' 或 'npm run tauri build'" -ForegroundColor Green
