# Verdent 插件防封补丁使用说明

## 补丁原理

Verdent 插件通过以下方式识别设备，用于封号检测：

| 标识类型 | 获取方式 | 补丁处理 |
|---------|---------|---------|
| Windows MachineGuid | 注册表 `HKLM\SOFTWARE\Microsoft\Cryptography` | ✅ 已禁用 |
| macOS IOPlatformExpertDevice | `ioreg` 命令 | ✅ 已禁用 |
| Linux machine-id | `/var/lib/dbus/machine-id` | ✅ 已禁用 |
| Verdent deviceId | VS Code globalStorage | ✅ 已替换为随机值 |
| machineIdSync/machineId | 系统API调用 | ✅ 已替换为随机值 |

## 使用方法

### 1. 应用补丁

```bash
cd "Vs code拓展"
python patch_extension_v2.py patch
```

### 2. 恢复原始文件

```bash
python patch_extension_v2.py restore
```

### 3. 指定文件路径

```bash
python patch_extension_v2.py patch "C:\path\to\extension.js"
```

## 补丁效果

应用补丁后：

1. **每次 VS Code 启动**时，插件会生成一个新的随机设备ID
2. **API 请求**中的 `device_id` 字段会使用随机值
3. **系统命令**获取机器码被禁用或替换

## 注意事项

1. **备份**：每次应用补丁都会自动创建备份文件 `extension.js.backup_YYYYMMDD_HHMMSS`
2. **插件更新**：Verdent 插件更新后需要重新应用补丁
3. **配合使用**：建议配合账号管理器的设备ID重置功能一起使用，效果更佳

## 补丁位置

补丁修改了 `extension/dist/extension.js` 文件中的以下函数：

1. `machineIdSync` - 同步获取机器ID
2. `machineId` - 异步获取机器ID  
3. `configManager.get("deviceId")` - 获取设备ID配置

## 已知限制

1. 补丁基于代码模式匹配，Verdent 大版本更新后可能需要调整
2. 服务端可能还有其他检测方式（如 IP、浏览器指纹等）
3. 建议配合代理使用以获得最佳效果

## 文件说明

```
Vs code拓展/
├── extension/
│   └── dist/
│       ├── extension.js           # 被补丁的文件
│       └── extension.js.backup_*  # 自动备份
├── patch_extension_v2.py          # 补丁工具（推荐）
├── patch_extension.py             # 补丁工具（旧版）
└── README_防封补丁.md             # 本说明文件
```

## 完整防封方案

为达到最佳防封效果，建议同时执行：

1. ✅ 应用插件补丁（本工具）
2. ✅ 使用账号管理器重置 VS Code 设备ID
3. ✅ 重置 Windows MachineGuid（如需要）
4. ✅ 清理 `~/.verdent` 目录
5. ✅ 清理 VS Code globalStorage 中的 verdent 数据
6. 🔲 使用不同 IP（代理）
