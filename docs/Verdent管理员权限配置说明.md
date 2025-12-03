# Verdent Account Manager 管理员权限配置说明

## 📋 概述

本文档说明如何为 Verdent Account Manager 配置管理员权限，使应用能够访问系统级目录和执行需要提升权限的操作。

## 🎯 为什么需要管理员权限

Verdent Account Manager 需要管理员权限来执行以下操作：

1. **访问系统目录**
   - `C:\Users\Administrator\.verdent\*`（Verdent 配置和数据目录）
   - `C:\Users\Administrator\AppData\Roaming\Code\User\globalStorage\*`（VS Code 全局存储）

2. **完全清理功能**
   - 清理 `C:\Users\Administrator\.verdent\projects`
   - 清理 `C:\Users\Administrator\.verdent\logs`
   - 清理 `C:\Users\Administrator\.verdent\checkpoints\checkpoints`
   - 清理 `C:\Users\Administrator\AppData\Roaming\Code\User\globalStorage\verdentai.verdent\tasks`
   - 清理 `C:\Users\Administrator\AppData\Roaming\Code\User\globalStorage\verdentai.verdent\checkpoints`
   - 重置 VS Code 设备 ID（telemetry.sqmId、telemetry.devDeviceId、telemetry.machineId）

3. **系统级操作**
   - 修改 Windows Credential Manager（凭据管理器）
   - 管理系统进程（重启 Verdent 客户端）
   - 创建开机自启动任务（可选）

## 🔧 实现方法

### 方法一：通过 build.rs 嵌入 Manifest（推荐）

这是最可靠的方法，在编译时将管理员权限清单嵌入到可执行文件中。

#### 已修改的文件：

1. **`Verdent_account_manger/src-tauri/build.rs`**
   - 添加了 Windows 特定的构建配置
   - 通过 `tauri_build::WindowsAttributes` 嵌入完整的 manifest
   - 支持通过环境变量 `REQUIRE_ADMIN` 控制是否需要管理员权限

2. **`Verdent_account_manger/src-tauri/verdent-account-manager.exe.manifest`**
   - 更新版本号为 `1.5.2.0`
   - 添加 `Microsoft.Windows.Common-Controls` 依赖
   - 设置 `requestedExecutionLevel` 为 `requireAdministrator`
   - 添加 Windows 兼容性声明（Vista 到 Windows 10/11）

3. **`Verdent_account_manger/src-tauri/tauri.conf.json`**
   - 在 `bundle.resources` 中添加了 `verdent-account-manager.exe.manifest`

### 方法二：使用外部工具（备用方案）

如果方法一不起作用，可以使用以下工具：

#### 使用 Resource Hacker
1. 下载 [Resource Hacker](http://www.angusj.com/resourcehacker/)
2. 打开编译后的 exe 文件
3. 添加 manifest 资源

#### 使用 Manifest Tool (mt.exe)
```cmd
"C:\Program Files (x86)\Windows Kits\10\bin\10.0.19041.0\x64\mt.exe" ^
  -manifest verdent-account-manager.exe.manifest ^
  -outputresource:verdent-account-manager.exe;1
```

## 📦 编译和构建

### 正常编译（需要管理员权限）

```bash
cd Verdent_account_manger
npm run tauri build
```

### 禁用管理员权限（开发测试）

如果在开发过程中不需要管理员权限，可以设置环境变量：

```powershell
# PowerShell
$env:REQUIRE_ADMIN="false"
npm run tauri dev
```

```cmd
# CMD
set REQUIRE_ADMIN=false
npm run tauri dev
```

## ✅ 验证管理员权限

### 1. 检查可执行文件属性
- 右键点击编译后的 `verdent-account-manager.exe`
- 选择"属性" > "兼容性"
- 应该看到"以管理员身份运行此程序"选项

### 2. 运行时检查
应用启动时会在控制台输出：
```
[*] 管理员权限检查: 是
```

### 3. UAC 提示
- 双击运行应用时，应该弹出 Windows UAC（用户账户控制）对话框
- 对话框会显示"你要允许此应用对你的设备进行更改吗？"
- 点击"是"后应用才会启动

### 4. 使用 Tauri 命令检查

前端可以调用以下命令检查权限：

```javascript
import { invoke } from '@tauri-apps/api/core'

// 检查是否以管理员权限运行
const isAdmin = await invoke('check_admin_privileges')
console.log('管理员权限:', isAdmin)
```

## 🔑 权限级别说明

Manifest 中的 `requestedExecutionLevel` 有三个选项：

1. **`asInvoker`**（默认）
   - 使用当前用户权限
   - 不会弹出 UAC 提示
   - 无法访问需要管理员权限的资源

2. **`highestAvailable`**（推荐用于可选权限）
   - 使用可用的最高权限
   - 如果用户是管理员，会弹出 UAC 提示
   - 如果用户不是管理员，使用普通用户权限

3. **`requireAdministrator`**（当前配置）
   - 始终需要管理员权限
   - 总是弹出 UAC 提示
   - 如果用户拒绝或不是管理员，应用无法启动

**当前配置**：使用 `requireAdministrator`，因为"完全清理"功能必须访问系统目录。

## 🚨 注意事项

1. **UAC 提示**
   - 每次启动应用都会弹出 UAC 对话框
   - 用户必须点击"是"才能运行
   - 这是 Windows 安全机制，无法绕过

2. **开发调试**
   - 开发时可以设置 `REQUIRE_ADMIN=false` 来禁用权限要求
   - 但某些功能（如完全清理）将无法正常工作

3. **用户体验**
   - 建议在应用启动时显示提示，说明为什么需要管理员权限
   - 可以在设置中添加选项，让用户选择是否启用需要管理员权限的功能

4. **安全性**
   - 以管理员权限运行的应用可以访问系统的任何部分
   - 确保代码安全，避免被恶意利用
   - 建议对敏感操作添加二次确认

## 📚 参考资料

- [Tauri Build Configuration](https://tauri.app/v1/api/config/#buildconfig)
- [Windows Application Manifest](https://docs.microsoft.com/en-us/windows/win32/sbscs/application-manifests)
- [UAC and Manifest Files](https://docs.microsoft.com/en-us/windows/security/identity-protection/user-account-control/how-user-account-control-works)

## 🔄 回滚方法

如果需要移除管理员权限要求：

1. 修改 `build.rs`，设置环境变量检查：
   ```rust
   let require_admin = env::var("REQUIRE_ADMIN").unwrap_or_else(|_| "false".to_string());
   ```

2. 或者直接修改 manifest 文件：
   ```xml
   <requestedExecutionLevel level="asInvoker" uiAccess="false"/>
   ```

3. 重新编译应用

