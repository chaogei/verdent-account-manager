import requests
import json
import uuid
import random
from typing import Optional, Dict, Any


def generate_device_id() -> str:
    """
    生成设备 ID（与 Verdent 插件相同的方式）
    插件使用随机 UUID v4 格式，而非真实硬件 ID
    
    Returns:
        UUID v4 格式的设备 ID，例如: a1b2c3d4-e5f6-4789-a012-3456789abcde
    """
    return str(uuid.uuid4())


def generate_device_id_custom() -> str:
    """
    使用插件中的自定义 UUID 生成算法
    格式: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
    
    Returns:
        自定义格式的 UUID 字符串
    """
    def replace_char(c):
        r = random.randint(0, 15)
        if c == 'x':
            return format(r, 'x')
        else:
            v = (r & 0x3) | 0x8
            return format(v, 'x')
    
    template = 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'
    return ''.join(replace_char(c) if c in 'xy' else c for c in template)


def generate_random_machine_id() -> str:
    """
    生成随机机器 ID（32位十六进制字符串）
    与插件中的 generateRandomMachineId 相同
    
    Returns:
        32位十六进制字符串
    """
    return ''.join(format(random.randint(0, 15), 'x') for _ in range(32))


class VerdentTrialAPI:
    """
    Verdent 免费试用订阅 API 客户端
    
    主要功能:
    - 获取用户信息（包括试用计划ID）
    - 获取免费试用订阅页面URL
    - 自动生成设备ID
    """
    
    def __init__(
        self,
        api_host: str = "https://api.verdent.ai",
        agent_host: str = "https://agent.verdent.ai",
        login_host: str = "https://login.verdent.ai",
        timeout: int = 30,
        debug: bool = False,
        proxy: Optional[str] = None
    ):
        """
        Args:
            api_host: API服务器地址
            agent_host: Agent服务器地址
            login_host: 登录服务器地址
            timeout: 请求超时时间（秒）
            debug: 是否开启调试模式
            proxy: 代理地址，格式如 'http://127.0.0.1:7890' 或 'socks5://127.0.0.1:1080'
        """
        self.api_host = api_host.rstrip('/')
        self.agent_host = agent_host.rstrip('/')
        self.login_host = login_host.rstrip('/')
        self.timeout = timeout
        self.auth_token: Optional[str] = None
        self.debug = debug
        self.proxy = proxy
        
        # 设置代理
        self.proxies = None
        if proxy:
            if proxy.startswith('socks'):
                # SOCKS代理需要 requests[socks] 支持
                # pip install requests[socks]
                self.proxies = {
                    'http': proxy,
                    'https': proxy
                }
            else:
                # HTTP/HTTPS代理
                self.proxies = {
                    'http': proxy,
                    'https': proxy
                }
            if debug:
                print(f"[DEBUG] 使用代理: {proxy}")
        
    def _make_request(
        self,
        method: str,
        url: str,
        data: Optional[Dict] = None,
        headers: Optional[Dict] = None,
        timeout: Optional[int] = None,
        debug: bool = False
    ) -> Dict[str, Any]:
        """发送 HTTP 请求并处理响应"""
        if headers is None:
            headers = {}
        
        headers.setdefault("Content-Type", "application/json")
        
        if self.auth_token and "Cookie" not in headers:
            headers["Cookie"] = f"token={self.auth_token}"
        
        if debug:
            print(f"\n[DEBUG] 请求详情:")
            print(f"  方法: {method}")
            print(f"  URL: {url}")
            print(f"  数据: {json.dumps(data, ensure_ascii=False, indent=2)}")
            print(f"  请求头: {json.dumps({k: v[:20]+'...' if k == 'Cookie' else v for k, v in headers.items()}, ensure_ascii=False, indent=2)}")
        
        try:
            response = requests.request(
                method=method,
                url=url,
                json=data,
                headers=headers,
                timeout=timeout or self.timeout,
                proxies=self.proxies,
                verify=True  # 可设为 False 跳过 SSL 验证（仅在测试环境使用）
            )
            
            if debug:
                print(f"\n[DEBUG] 响应状态: {response.status_code}")
                print(f"[DEBUG] 响应头: {dict(response.headers)}")
                try:
                    print(f"[DEBUG] 响应内容: {json.dumps(response.json(), ensure_ascii=False, indent=2)}")
                except:
                    print(f"[DEBUG] 响应内容 (非JSON): {response.text[:500]}")
            
            response.raise_for_status()
            
            result = response.json()
            
            if "errCode" in result:
                if result["errCode"] != 0:
                    raise Exception(f"API Error (errCode={result['errCode']}): {result.get('errMsg', 'Unknown error')}")
                return result.get("data", {})
            
            if "data" in result:
                return result["data"]
            
            return result
            
        except requests.exceptions.HTTPError as e:
            error_msg = f"HTTP Error {response.status_code}: {response.reason}"
            try:
                error_detail = response.json()
                error_msg += f"\n详细信息: {json.dumps(error_detail, ensure_ascii=False, indent=2)}"
            except:
                error_msg += f"\n响应内容: {response.text[:200]}"
            raise Exception(error_msg)
        except requests.exceptions.RequestException as e:
            raise Exception(f"Request failed: {str(e)}")
    
    def pkce_callback(self, code: str, code_verifier: str) -> str:
        """
        PKCE 认证回调
        
        Args:
            code: 授权码
            code_verifier: PKCE code verifier
            
        Returns:
            认证令牌
        """
        url = f"{self.login_host}/passport/pkce/callback"
        data = {
            "code": code,
            "codeVerifier": code_verifier
        }
        
        result = self._make_request("POST", url, data=data, timeout=100)
        self.auth_token = result.get("token")
        return self.auth_token
    
    def get_user_info(self) -> Dict[str, Any]:
        """
        获取用户信息（包含 subscriptionBonus 和 trialPlanId）
        
        Returns:
            用户信息字典，包含:
            - trialPlanId: 试用计划 ID
            - subscriptionBonus: 订阅奖励信息 (title, description, tips)
            - email: 用户邮箱
            - isLogin: 是否已登录
            等等
        """
        if not self.auth_token:
            raise Exception("未设置认证令牌，请先调用 set_auth_token() 或 pkce_callback()")
        
        url = f"{self.agent_host}/user/center/info"
        return self._make_request("GET", url, timeout=10, debug=self.debug)
    
    def create_subscription(
        self,
        plan_id: str,
        device_id: Optional[str] = None,
        source: str = "verdent"
    ) -> Dict[str, Any]:
        """
        创建订阅并获取 checkout URL
        
        Args:
            plan_id: 计划 ID（可从 get_user_info() 的 trialPlanId 获取）
            device_id: 设备 ID（如果为 None，将自动生成）
            source: 来源标识
            
        Returns:
            包含 checkout_url 的字典
        """
        if not self.auth_token:
            raise Exception("未设置认证令牌，请先调用 set_auth_token() 或 pkce_callback()")
        
        if device_id is None:
            device_id = generate_device_id()
        
        url = f"{self.api_host}/verdent/subscription/create"
        data = {
            "plan_id": plan_id,
            "device_id": device_id,
            "source": source
        }
        
        return self._make_request("POST", url, data=data, debug=self.debug)
    
    def set_auth_token(self, token: str):
        """设置认证令牌"""
        self.auth_token = token
    
    def get_free_trial_page(self, device_id: Optional[str] = None, force: bool = True) -> str:
        """
        获取免费试用页面 URL
        这是完整的流程方法，相当于前端的 getFreeTrialPage 处理器
        
        Args:
            device_id: 设备 ID（如果为 None，将自动生成）
            force: 是否强制尝试，忽略 isTrialAvailable 检查（默认 True）
            
        Returns:
            checkout_url: 试用页面的 URL
        """
        if not self.auth_token:
            raise Exception("未设置认证令牌，请先调用 set_auth_token() 或 pkce_callback()")
        
        user_info = self.get_user_info()
        
        if not force:
            is_trial_available = user_info.get("isTrialAvailable", False)
            if not is_trial_available:
                raise Exception(
                    "该账户的免费试用已不可用。\n"
                    "可能原因：\n"
                    "  1. 该账户已经使用过免费试用\n"
                    "  2. 该账户已有订阅计划\n"
                    "  3. 该账户不符合试用资格\n"
                    f"账户状态: isSubscribe={user_info.get('isSubscribe')}, "
                    f"isTrialAvailable={is_trial_available}\n"
                    "提示: 使用 force=True 参数可强制尝试"
                )
        
        trial_plan_id = user_info.get("trialPlanId")
        if not trial_plan_id:
            raise Exception("用户信息中未找到 trialPlanId")
        
        if device_id is None:
            device_id = generate_device_id()
        
        result = self.create_subscription(
            plan_id=trial_plan_id,
            device_id=device_id,
            source="verdent"
        )
        
        checkout_url = result.get("checkout_url")
        if not checkout_url:
            raise Exception("未能获取 checkout_url")
        
        return checkout_url
    


def main():
    """
    示例：获取免费试用订阅
    
    流程:
    1. 设置认证令牌
    2. 获取用户信息
    3. 获取免费试用页面URL
    """
    # 配置代理（根据需要修改）
    # 示例: HTTP代理 'http://127.0.0.1:7890'
    # 示例: SOCKS5代理 'socks5://127.0.0.1:1080'
    proxy = "http://127.0.0.1:7890"  # 设置为 None 表示不使用代理
    # proxy = 'http://127.0.0.1:7890'  # 取消注释以启用代理
    
    api = VerdentTrialAPI(
        debug=False,
        proxy=proxy
    )
    
    if proxy:
        print(f"正在使用代理: {proxy}\n")
    
    # TODO: 替换为实际的认证令牌
    auth_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VyX2lkIjo3NDk0NjU2MDQzOTc2NDU4MjQsInZlcnNpb24iOjAsInRva2VuX3R5cGUiOiJhY2Nlc3MiLCJleHAiOjE3NjU4NzcxNTcsImlhdCI6MTc2MzI4NTE1NywibmJmIjoxNzYzMjg1MTU3fQ.3g8U-Fhs8UljRQ0_PyD--YBHkMH49S37ZfJv9YJhzD0"
    api.set_auth_token(auth_token)
    
    device_id = generate_device_id()
    print(f"生成的设备 ID (UUID v4): {device_id}")
    
    device_id_custom = generate_device_id_custom()
    print(f"生成的设备 ID (自定义): {device_id_custom}")
    
    machine_id = generate_random_machine_id()
    print(f"生成的机器 ID (32位): {machine_id}\n")
    
    print("=" * 60)
    print("1. 获取用户信息...")
    print("=" * 60)
    try:
        user_info = api.get_user_info()
        print(f"✓ 用户邮箱: {user_info.get('email')}")
        print(f"✓ 试用计划 ID: {user_info.get('trialPlanId')}")
        print(f"✓ 是否已订阅: {user_info.get('isSubscribe')}")
        print(f"✓ 试用是否可用: {user_info.get('isTrialAvailable')}")
        print(f"✓ 订阅奖励: {user_info.get('subscriptionBonus')}")
    except Exception as e:
        print(f"✗ 错误: {e}")
        return
    
    print("\n" + "=" * 60)
    print("2. 尝试获取免费试用页面...")
    print("=" * 60)
    try:
        checkout_url = api.get_free_trial_page(device_id=device_id)
        print(f"✓ 试用页面 URL: {checkout_url}")
        print("\n提示: 这是免费试用订阅页面，新用户可以获得免费积分")
    except Exception as e:
        print(f"✗ {e}")
        if "Connection" in str(e) or "timeout" in str(e):
            print("\n提示: 如果连接失败，请检查：")
            print("  1. 是否需要配置代理（修改 proxy 变量）")
            print("  2. 代理服务是否正常运行")
            print("  3. 网络连接是否正常")


if __name__ == "__main__":
    main()
