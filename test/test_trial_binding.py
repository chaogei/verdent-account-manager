#!/usr/bin/env python3
"""
测试绑卡功能是否正常工作

功能测试：
1. 代理设置保存和读取
2. 获取绑卡链接
3. 打开无痕浏览器
"""

import json
import requests
import sys
import os

# 添加项目路径
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from Verdent_API.verdent_trial_api import VerdentTrialAPI

def test_trial_binding():
    """测试绑卡功能"""
    
    # 配置
    proxy_url = "http://127.0.0.1:7890"  # 根据实际情况修改
    test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."  # 需要替换为有效的token
    
    print("=" * 60)
    print("测试 Verdent 绑卡功能")
    print("=" * 60)
    
    # 创建API实例
    api = VerdentTrialAPI(
        debug=True,
        proxy=proxy_url
    )
    
    # 设置token
    api.set_auth_token(test_token)
    
    try:
        # 1. 获取用户信息
        print("\n1. 获取用户信息...")
        user_info = api.get_user_info()
        print(f"✓ 用户邮箱: {user_info.get('email')}")
        print(f"✓ 试用计划ID: {user_info.get('trialPlanId')}")
        print(f"✓ 是否已订阅: {user_info.get('isSubscribe')}")
        print(f"✓ 试用是否可用: {user_info.get('isTrialAvailable')}")
        
        # 2. 获取绑卡链接
        print("\n2. 获取绑卡链接...")
        checkout_url = api.get_free_trial_page(force=True)
        print(f"✓ 绑卡链接: {checkout_url}")
        
        # 3. 测试结果
        print("\n" + "=" * 60)
        print("测试结果: ✅ 成功")
        print("=" * 60)
        print("\n说明:")
        print("1. 绑卡链接已成功获取")
        print("2. 可以在账号管理器中点击绑卡按钮打开此链接")
        print("3. 链接将在无痕浏览器中打开以保护隐私")
        
    except Exception as e:
        print(f"\n❌ 测试失败: {e}")
        print("\n可能的原因:")
        print("1. Token 已过期，需要重新获取")
        print("2. 代理设置不正确")
        print("3. 该账户已经使用过免费试用")
        return False
    
    return True

def test_proxy_settings():
    """测试代理设置功能"""
    print("\n" + "=" * 60)
    print("测试代理设置功能")
    print("=" * 60)
    
    # 测试代理配置文件路径
    import platform
    
    if platform.system() == "Windows":
        config_dir = os.path.join(os.environ.get("APPDATA", ""), "ai.verdent.account-manager")
    elif platform.system() == "Darwin":  # macOS
        config_dir = os.path.expanduser("~/Library/Application Support/ai.verdent.account-manager")
    else:  # Linux
        config_dir = os.path.expanduser("~/.config/ai.verdent.account-manager")
    
    proxy_settings_file = os.path.join(config_dir, "proxy_settings.json")
    
    # 测试保存代理设置
    test_settings = {
        "enabled": True,
        "url": "http://127.0.0.1:7890"
    }
    
    # 确保目录存在
    os.makedirs(config_dir, exist_ok=True)
    
    # 保存设置
    with open(proxy_settings_file, "w", encoding="utf-8") as f:
        json.dump(test_settings, f, indent=2)
    
    print(f"✓ 代理设置已保存到: {proxy_settings_file}")
    
    # 读取设置
    with open(proxy_settings_file, "r", encoding="utf-8") as f:
        loaded_settings = json.load(f)
    
    print(f"✓ 代理设置已读取: {loaded_settings}")
    
    if loaded_settings == test_settings:
        print("✓ 代理设置读写测试通过")
        return True
    else:
        print("❌ 代理设置读写测试失败")
        return False

if __name__ == "__main__":
    # 运行测试
    print("开始测试绑卡功能...")
    
    # 测试代理设置
    proxy_test_passed = test_proxy_settings()
    
    # 提示用户输入token进行完整测试
    print("\n" + "=" * 60)
    print("完整功能测试")
    print("=" * 60)
    print("\n要进行完整的绑卡功能测试，请提供有效的 Verdent Token")
    print("获取方法：")
    print("1. 登录 Verdent 网站")
    print("2. 打开浏览器开发者工具 (F12)")
    print("3. 在 Application/Storage -> Cookies 中找到 token")
    print("\n或者跳过此测试 (输入 'skip')")
    
    user_input = input("\n请输入 Token (或 'skip'): ").strip()
    
    if user_input.lower() != 'skip' and len(user_input) > 50:
        # 替换测试文件中的token并运行测试
        import tempfile
        temp_test = """
# 临时测试脚本
from Verdent_API.verdent_trial_api import VerdentTrialAPI

api = VerdentTrialAPI(debug=False, proxy="http://127.0.0.1:7890")
api.set_auth_token("{}")
try:
    url = api.get_free_trial_page(force=True)
    print("✅ 绑卡链接获取成功:", url)
except Exception as e:
    print("❌ 获取失败:", e)
""".format(user_input)
        
        exec(temp_test)
    else:
        print("\n跳过完整功能测试")
        print("代理设置测试结果:", "✅ 通过" if proxy_test_passed else "❌ 失败")
    
    print("\n测试完成！")
