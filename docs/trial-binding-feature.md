# Verdent 账号管理器 - 绑卡功能使用指南

## 功能概述

新增的绑卡功能允许用户快速获取 Verdent AI 的免费试用，并自动在无痕浏览器中打开绑卡页面。

## 主要特性

1. **一键绑卡** - 在每个账户卡片上添加绑卡按钮，点击即可获取试用链接
2. **代理支持** - 支持配置 HTTP/HTTPS/SOCKS5 代理，适用于网络受限环境
3. **隐私保护** - 自动在无痕/隐私浏览模式下打开绑卡页面
4. **智能检测** - 自动检测账户是否符合试用条件

## 使用步骤

### 1. 配置代理（可选）

如果需要使用代理访问 Verdent API：

1. 点击右上角的 **设置** 图标
2. 在代理设置对话框中：
   - 勾选"启用代理"
   - 输入代理地址（例如：`http://127.0.0.1:7890`）
   - 点击"保存"

### 2. 获取绑卡链接

1. 在账户列表中找到需要绑卡的账户
2. 确保该账户已有有效的 Token（如没有，先点击"密码登录"获取）
3. 点击账户卡片上的 **绑卡** 按钮（在刷新按钮右侧）
4. 系统会自动：
   - 获取用户信息
   - 检查试用资格
   - 生成绑卡链接
   - 在无痕浏览器中打开

### 3. 完成绑卡

在打开的浏览器页面中：

1. 填写支付信息（不会立即扣费）
2. 完成验证
3. 确认订阅试用计划

## 技术实现

### 前端部分

#### AccountCard.vue
- 新增绑卡按钮和处理函数
- 调用后端 API 获取绑卡链接
- 显示加载状态和错误提示

#### App.vue
- 添加代理设置对话框
- 管理代理配置状态
- 保存和加载代理设置

### 后端部分

#### proxy_manager.rs
- 代理设置的持久化存储
- 配置文件管理

#### commands.rs
新增命令：
- `get_proxy_settings` - 获取代理配置
- `save_proxy_settings` - 保存代理配置
- `get_trial_checkout_url` - 获取试用绑卡链接
- `open_incognito_browser` - 打开无痕浏览器

### API 集成

基于 `verdent_trial_api.py` 的功能，后端实现了：

1. **用户信息获取**
   - 端点：`https://agent.verdent.ai/user/center/info`
   - 获取 trialPlanId 和试用状态

2. **创建订阅**
   - 端点：`https://api.verdent.ai/verdent/subscription/create`
   - 参数：plan_id, device_id, source
   - 返回：checkout_url

3. **设备ID生成**
   - 使用 UUID v4 格式
   - 每次请求生成新的设备ID

## 浏览器支持

### Windows
- Chrome（优先）- 无痕模式
- Edge - InPrivate 模式
- 默认浏览器（后备选项）

### macOS
- Safari - 私密浏览模式
- Chrome - 无痕模式（如已安装）

### Linux
- Chrome/Chromium - 无痕模式
- Firefox - 隐私窗口
- xdg-open（后备选项）

## 代理配置

### 支持的代理类型
- HTTP 代理：`http://host:port`
- HTTPS 代理：`https://host:port`
- SOCKS5 代理：`socks5://host:port`

### 配置文件位置
- Windows：`%APPDATA%\ai.verdent.account-manager\proxy_settings.json`
- macOS：`~/Library/Application Support/ai.verdent.account-manager/proxy_settings.json`
- Linux：`~/.config/ai.verdent.account-manager/proxy_settings.json`

### 配置格式
```json
{
  "enabled": true,
  "url": "http://127.0.0.1:7890"
}
```

## 注意事项

1. **试用限制**
   - 每个账户只能使用一次免费试用
   - 提供 100 积分，有效期 7 天
   - 不会自动扣费

2. **Token 要求**
   - 必须先获取有效的 Token 才能使用绑卡功能
   - Token 过期需要重新登录获取

3. **网络要求**
   - 需要能访问 Verdent API 服务器
   - 如有网络限制，请配置代理

4. **隐私保护**
   - 绑卡页面在无痕模式下打开
   - 不会保存浏览历史和 Cookie
   - 关闭窗口后不留痕迹

## 故障排除

### 问题：获取绑卡链接失败

可能原因：
- Token 已过期 → 重新登录获取新 Token
- 网络连接问题 → 检查网络或配置代理
- 账户已使用过试用 → 该账户无法再次获取试用

### 问题：无法打开浏览器

可能原因：
- 浏览器未安装 → 安装 Chrome 或 Edge
- 权限不足 → 以管理员身份运行
- 系统限制 → 手动复制链接到浏览器打开

### 问题：代理无法连接

可能原因：
- 代理地址错误 → 检查代理配置
- 代理服务未启动 → 确保代理软件正在运行
- 防火墙阻止 → 添加防火墙例外

## 测试

运行测试脚本验证功能：

```bash
python test/test_trial_binding.py
```

测试内容：
- 代理设置保存和读取
- API 连接测试
- 绑卡链接获取
- 浏览器打开测试

## 更新日志

### v1.3.0 (2024-11)
- ✨ 新增绑卡功能
- ✨ 添加代理设置支持
- ✨ 实现无痕浏览器打开
- 🔧 集成 verdent_trial_api 功能
- 🎨 优化UI交互体验
- 🐛 修复CSS空规则集警告
