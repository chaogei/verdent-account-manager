# macOS 资源文件打包修复

## 问题描述

在 macOS 上运行打包后的 Verdent Account Manager 应用时，自动注册功能失败，错误信息：
```
[×] 未找到任何可执行文件!
```

经过调试发现，应用的 Resources 目录下没有包含必要的 Python 脚本文件。

## 根本原因

1. **Tauri 配置问题**：`tauri.conf.json` 中的资源路径配置不一致
   - 原配置使用 `../../verdent_auto_register.py` 指向项目根目录
   - 但 GitHub Actions 将文件复制到了 `resources` 子目录

2. **资源文件未打包**：macOS 构建过程中，Python 脚本文件没有被正确包含到 `.app` 包的 Resources 目录

## 解决方案

### 1. 统一资源路径配置

修改 `Verdent_account_manger/src-tauri/tauri.conf.json`：
```json
"resources": [
  "../resources/verdent_auto_register.exe",      // Windows 可执行文件
  "../resources/verdent_auto_register.py",       // Python 主脚本
  "../resources/verdent_auto_register_wrapper.py" // Python 包装脚本
]
```

所有平台统一使用 `resources` 目录作为资源存放位置。

### 2. 添加资源准备脚本

创建了两个脚本来准备资源文件：
- `prepare_resources.ps1` - Windows PowerShell 脚本
- `prepare_resources.sh` - macOS/Linux Shell 脚本

这些脚本会：
1. 创建 `Verdent_account_manger/resources` 目录
2. 复制 Python 脚本到该目录
3. 设置适当的执行权限（macOS/Linux）

### 3. 更新构建脚本

更新了构建脚本，在打包前自动准备资源文件：
- `build_tauri.ps1` - 添加了资源准备步骤
- `build_all.ps1` - 添加了 Python 脚本复制步骤

### 4. GitHub Actions 配置

GitHub Actions 工作流已经正确配置，会在构建前准备资源文件：
```yaml
- name: 准备 Python 脚本资源 (非 Windows)
  if: matrix.platform != 'windows-latest'
  run: |
    mkdir -p Verdent_account_manger/resources
    cp verdent_auto_register.py Verdent_account_manger/resources/
    cp verdent_auto_register_wrapper.py Verdent_account_manger/resources/
    chmod +x Verdent_account_manger/resources/*.py
```

## 使用说明

### 本地开发

在构建前运行资源准备脚本：

**Windows:**
```powershell
.\prepare_resources.ps1
# 或直接运行构建脚本（已包含资源准备）
.\build_tauri.ps1
```

**macOS/Linux:**
```bash
./prepare_resources.sh
# 或直接运行构建脚本
npm run tauri build
```

### CI/CD 构建

GitHub Actions 工作流会自动处理资源准备，无需手动干预。

## 验证

构建完成后，检查资源是否正确打包：

**macOS:**
```bash
# 查看 .app 包内容
ls -la "/Applications/Verdent Account Manager.app/Contents/Resources/"
# 应该包含:
# - verdent_auto_register.py
# - verdent_auto_register_wrapper.py
```

**Windows:**
安装后检查安装目录下的 resources 文件夹。

## 注意事项

1. **跨平台兼容**：虽然 Windows 使用 .exe 文件，macOS/Linux 使用 .py 脚本，但配置文件统一包含所有资源，Tauri 会根据平台选择合适的文件。

2. **Python 依赖**：macOS/Linux 用户需要安装 Python 3 和相关依赖。包装脚本 `verdent_auto_register_wrapper.py` 会自动检查并提示安装。

3. **文件权限**：macOS/Linux 上的 Python 脚本需要执行权限，构建脚本会自动设置。

## 相关文件

- `Verdent_account_manger/src-tauri/tauri.conf.json` - Tauri 配置
- `prepare_resources.ps1` - Windows 资源准备脚本
- `prepare_resources.sh` - macOS/Linux 资源准备脚本
- `.github/workflows/build.yml` - CI/CD 构建配置
