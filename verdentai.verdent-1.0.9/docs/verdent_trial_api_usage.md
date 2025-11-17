# Verdent Limited-Time Free Trial API 使用说明

## 概述

本 Python 脚本提取了 Verdent 插件中与"Limited-Time Free Trial"相关的 API 方法，封装为易于使用的 Python 类。

## API 端点

该 API 涉及以下三个主要端点：

| 端点 | 方法 | 服务器 | 用途 |
|------|------|--------|------|
| `/passport/pkce/callback` | POST | `https://login.verdent.ai` | PKCE 认证回调 |
| `/user/center/info` | GET | `https://agent.verdent.ai` | 获取用户信息（包含试用计划信息） |
| `/verdent/subscription/create` | POST | `https://api.verdent.ai` | 创建订阅，获取 checkout_url |

## 安装依赖

```bash
pip install requests
```

## 快速开始

### 1. 基本使用

```python
from verdent_trial_api import VerdentTrialAPI

# 创建 API 客户端
api = VerdentTrialAPI()

# 设置认证令牌（从浏览器或其他方式获取）
api.set_auth_token("your_auth_token_here")

# 获取免费试用页面 URL
try:
    checkout_url = api.get_free_trial_page(device_id="my-device-001")
    print(f"试用页面: {checkout_url}")
except Exception as e:
    print(f"错误: {e}")
```

### 2. 分步操作

```python
from verdent_trial_api import VerdentTrialAPI

api = VerdentTrialAPI()
api.set_auth_token("your_auth_token_here")

# 步骤 1: 获取用户信息
user_info = api.get_user_info()
print(f"用户邮箱: {user_info['email']}")
print(f"试用计划 ID: {user_info['trialPlanId']}")
print(f"订阅奖励信息: {user_info['subscriptionBonus']}")

# 步骤 2: 创建订阅
result = api.create_subscription(
    plan_id=user_info['trialPlanId'],
    device_id="my-device-001"
)
print(f"Checkout URL: {result['checkout_url']}")
```

### 3. 使用 PKCE 认证

```python
from verdent_trial_api import VerdentTrialAPI

api = VerdentTrialAPI()

# 使用 PKCE 认证流程获取令牌
auth_code = "authorization_code_from_oauth"
code_verifier = "your_code_verifier"

token = api.pkce_callback(auth_code, code_verifier)
print(f"获取到令牌: {token}")

# 现在可以使用其他 API 方法
user_info = api.get_user_info()
```

## 类方法详解

### `VerdentTrialAPI`

#### 初始化参数

```python
VerdentTrialAPI(
    api_host: str = "https://api.verdent.ai",
    agent_host: str = "https://agent.verdent.ai",
    login_host: str = "https://login.verdent.ai",
    timeout: int = 30
)
```

- `api_host`: API 服务器地址
- `agent_host`: Agent 服务器地址
- `login_host`: 登录服务器地址
- `timeout`: 请求超时时间（秒）

#### 主要方法

##### `set_auth_token(token: str)`

设置认证令牌。

**参数:**
- `token`: 认证令牌字符串

**示例:**
```python
api.set_auth_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...")
```

---

##### `pkce_callback(code: str, code_verifier: str) -> str`

执行 PKCE 认证回调，获取认证令牌。

**参数:**
- `code`: 授权码
- `code_verifier`: PKCE code verifier

**返回:**
- 认证令牌字符串

**示例:**
```python
token = api.pkce_callback("auth_code_123", "verifier_456")
```

---

##### `get_user_info() -> Dict[str, Any]`

获取用户信息，包含试用计划和订阅奖励信息。

**返回字典包含:**
- `trialPlanId`: 试用计划 ID
- `subscriptionBonus`: 订阅奖励信息
  - `title`: 标题（默认 "Limited-Time Free Trial"）
  - `description`: 描述
  - `tips`: 提示信息
- `email`: 用户邮箱
- `isLogin`: 是否已登录
- `tokenInfo`: 令牌信息
- `subscriptionInfo`: 订阅信息

**示例:**
```python
user_info = api.get_user_info()
print(user_info['subscriptionBonus']['title'])
```

---

##### `create_subscription(plan_id: str, device_id: str = "unknown-device", source: str = "verdent") -> Dict[str, Any]`

创建订阅并获取 checkout URL。

**参数:**
- `plan_id`: 计划 ID（通常从 `get_user_info()` 获取）
- `device_id`: 设备 ID（默认 "unknown-device"）
- `source`: 来源标识（默认 "verdent"）

**返回:**
- 包含 `checkout_url` 的字典

**示例:**
```python
result = api.create_subscription(
    plan_id="trial_plan_123",
    device_id="laptop-001",
    source="verdent"
)
print(result['checkout_url'])
```

---

##### `get_free_trial_page(device_id: str = "unknown-device") -> str`

一键获取免费试用页面 URL（完整流程）。

这个方法会自动：
1. 获取用户信息
2. 提取试用计划 ID
3. 创建订阅
4. 返回 checkout URL

**参数:**
- `device_id`: 设备 ID（默认 "unknown-device"）

**返回:**
- checkout URL 字符串

**示例:**
```python
checkout_url = api.get_free_trial_page("my-device-001")
print(f"请访问: {checkout_url}")
```

## 认证令牌获取

### 方法 1: 从浏览器获取

1. 打开浏览器开发者工具（F12）
2. 访问 Verdent 相关页面并登录
3. 查看 Network 标签
4. 找到请求的 Cookie 中的 `token` 值

### 方法 2: 从 VSCode 扩展存储获取

认证令牌存储在 VSCode 扩展的 secret storage 中，键名为 `ycAuthToken`。

## 错误处理

```python
from verdent_trial_api import VerdentTrialAPI

api = VerdentTrialAPI()

try:
    api.set_auth_token("your_token")
    checkout_url = api.get_free_trial_page()
    print(f"成功: {checkout_url}")
    
except Exception as e:
    if "未设置认证令牌" in str(e):
        print("请先设置认证令牌")
    elif "trialPlanId" in str(e):
        print("用户没有可用的试用计划")
    elif "Request failed" in str(e):
        print("网络请求失败，请检查网络连接")
    else:
        print(f"未知错误: {e}")
```

## 常见错误

| 错误消息 | 原因 | 解决方法 |
|---------|------|---------|
| `未设置认证令牌` | 没有调用 `set_auth_token()` | 先设置有效的认证令牌 |
| `用户信息中未找到 trialPlanId` | 用户账户没有试用计划 | 检查用户账户状态 |
| `API Error: ...` | API 返回错误码 | 查看具体错误信息，可能是权限或参数问题 |
| `Request failed: ...` | 网络请求失败 | 检查网络连接和服务器地址 |

## 响应数据示例

### `get_user_info()` 响应

```json
{
  "email": "user@example.com",
  "isLogin": true,
  "trialPlanId": "plan_abc123xyz",
  "subscriptionBonus": {
    "title": "Limited-Time Free Trial",
    "description": "Get 7 days of premium features",
    "tips": "Start your free trial now"
  },
  "tokenInfo": {
    "tokenConsumed": 1000,
    "tokenFlexibleLeft": 9000
  },
  "subscriptionInfo": {
    "levelName": "Premium"
  }
}
```

### `create_subscription()` 响应

```json
{
  "checkout_url": "https://verdent.ai/checkout/session_xyz789"
}
```

## 自定义服务器地址

如果使用自部署的 Verdent 服务器：

```python
api = VerdentTrialAPI(
    api_host="https://api.your-domain.com",
    agent_host="https://agent.your-domain.com",
    login_host="https://login.your-domain.com"
)
```

## 完整示例

```python
from verdent_trial_api import VerdentTrialAPI
import webbrowser

def start_free_trial(auth_token: str, device_id: str = "python-client"):
    """启动免费试用流程"""
    
    api = VerdentTrialAPI()
    api.set_auth_token(auth_token)
    
    try:
        print("正在获取用户信息...")
        user_info = api.get_user_info()
        print(f"✓ 用户: {user_info['email']}")
        
        bonus = user_info.get('subscriptionBonus', {})
        print(f"✓ 试用标题: {bonus.get('title', 'N/A')}")
        print(f"✓ 试用描述: {bonus.get('description', 'N/A')}")
        
        print("\n正在创建试用订阅...")
        checkout_url = api.get_free_trial_page(device_id=device_id)
        print(f"✓ 获取到 checkout URL")
        
        print(f"\n试用页面: {checkout_url}")
        
        open_browser = input("\n是否在浏览器中打开? (y/n): ")
        if open_browser.lower() == 'y':
            webbrowser.open(checkout_url)
            print("✓ 已在浏览器中打开")
        
        return checkout_url
        
    except Exception as e:
        print(f"✗ 错误: {e}")
        return None

if __name__ == "__main__":
    token = input("请输入认证令牌: ").strip()
    device_id = input("请输入设备 ID (直接回车使用默认值): ").strip()
    
    if not device_id:
        device_id = "python-client"
    
    start_free_trial(token, device_id)
```

## 技术细节

### API 请求格式

所有请求使用以下标准格式：

**请求头:**
```
Content-Type: application/json
Cookie: token=<auth_token>
```

**响应格式:**
```json
{
  "errCode": 0,
  "errMsg": "success",
  "data": { ... }
}
```

### 超时设置

- PKCE 回调: 100 秒
- 用户信息获取: 10 秒
- 创建订阅: 30 秒（默认）

## 注意事项

1. **认证令牌安全**: 请妥善保管认证令牌，不要在代码中硬编码或泄露
2. **请求频率**: 避免频繁请求，建议添加适当的延迟和重试机制
3. **错误处理**: 生产环境中应完善错误处理和日志记录
4. **设备 ID**: 建议使用唯一且有意义的设备标识符

## 许可证

本脚本基于 Verdent 插件源代码提取，使用时请遵守相关许可协议。

## 支持

如有问题，请联系: hi@verdent.ai
