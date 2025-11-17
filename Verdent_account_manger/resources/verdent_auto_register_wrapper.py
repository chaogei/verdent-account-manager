#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Verdent AI 自动注册脚本启动器
自动检测并安装依赖，然后运行主脚本
"""

import sys
import os
import subprocess
import platform
from pathlib import Path

def check_python_version():
    """检查 Python 版本"""
    version = sys.version_info
    if version.major < 3 or (version.major == 3 and version.minor < 7):
        print(f"❌ Python 版本过低: {sys.version}")
        print("需要 Python 3.7 或更高版本")
        print("请访问 https://www.python.org 下载最新版本")
        return False
    print(f"✓ Python 版本: {sys.version}")
    return True

def check_pip():
    """检查 pip 是否可用"""
    try:
        import pip
        print("✓ pip 已安装")
        return True
    except ImportError:
        print("❌ pip 未安装")
        print("尝试安装 pip...")
        
        # 下载 get-pip.py
        try:
            import urllib.request
            urllib.request.urlretrieve(
                'https://bootstrap.pypa.io/get-pip.py', 
                'get-pip.py'
            )
            subprocess.check_call([sys.executable, 'get-pip.py'])
            os.remove('get-pip.py')
            print("✓ pip 安装成功")
            return True
        except Exception as e:
            print(f"❌ pip 安装失败: {e}")
            print("\n请手动安装 pip:")
            print("1. 下载 https://bootstrap.pypa.io/get-pip.py")
            print(f"2. 运行: {sys.executable} get-pip.py")
            return False

def get_pip_command():
    """获取正确的 pip 命令"""
    # 尝试不同的 pip 命令
    commands = [
        [sys.executable, '-m', 'pip'],
        ['pip3'],
        ['pip'],
    ]
    
    for cmd in commands:
        try:
            result = subprocess.run(
                cmd + ['--version'],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode == 0:
                return cmd
        except:
            continue
    
    return None

def check_and_install_package(package_name, import_name=None):
    """检查并安装包"""
    if import_name is None:
        import_name = package_name
    
    # 检查是否已安装
    try:
        __import__(import_name)
        print(f"✓ {package_name} 已安装")
        return True
    except ImportError:
        print(f"⚠ {package_name} 未安装，尝试自动安装...")
        
        pip_cmd = get_pip_command()
        if not pip_cmd:
            print("❌ 无法找到可用的 pip 命令")
            return False
        
        try:
            # 尝试安装包
            print(f"执行: {' '.join(pip_cmd + ['install', package_name])}")
            result = subprocess.run(
                pip_cmd + ['install', package_name],
                capture_output=True,
                text=True,
                timeout=300  # 5分钟超时
            )
            
            if result.returncode == 0:
                # 再次检查是否成功导入
                try:
                    __import__(import_name)
                    print(f"✓ {package_name} 安装成功")
                    return True
                except ImportError:
                    print(f"❌ {package_name} 安装后仍无法导入")
                    return False
            else:
                print(f"❌ {package_name} 安装失败")
                print(f"错误信息: {result.stderr}")
                return False
                
        except subprocess.TimeoutExpired:
            print(f"❌ 安装 {package_name} 超时")
            return False
        except Exception as e:
            print(f"❌ 安装 {package_name} 时出错: {e}")
            return False

def check_chrome():
    """检查 Chrome 浏览器是否安装"""
    system = platform.system()
    
    # 不同系统的 Chrome 路径
    chrome_paths = []
    
    if system == "Windows":
        chrome_paths = [
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
            os.path.expandvars(r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe"),
        ]
    elif system == "Darwin":  # macOS
        chrome_paths = [
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            "/Applications/Chromium.app/Contents/MacOS/Chromium",
        ]
    elif system == "Linux":
        chrome_paths = [
            "/usr/bin/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/chromium",
            "/usr/bin/chromium-browser",
            "/snap/bin/chromium",
        ]
    
    # 检查路径
    for path in chrome_paths:
        if os.path.exists(path):
            print(f"✓ 找到 Chrome: {path}")
            return True
    
    # 尝试使用 which/where 命令
    try:
        cmd = "where" if system == "Windows" else "which"
        browsers = ["google-chrome", "google-chrome-stable", "chromium", "chrome"]
        
        for browser in browsers:
            result = subprocess.run(
                [cmd, browser],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                print(f"✓ 找到 Chrome: {browser}")
                return True
    except:
        pass
    
    print("⚠ 未找到 Chrome 浏览器")
    print("\n请安装 Chrome 浏览器:")
    
    if system == "Windows":
        print("访问: https://www.google.com/chrome/")
    elif system == "Darwin":
        print("运行: brew install --cask google-chrome")
        print("或访问: https://www.google.com/chrome/")
    elif system == "Linux":
        print("Ubuntu/Debian: sudo apt install google-chrome-stable")
        print("或访问: https://www.google.com/chrome/")
    
    return False

def install_dependencies():
    """安装所有依赖"""
    print("=" * 60)
    print("检查和安装依赖")
    print("=" * 60)
    
    # 检查 Python 版本
    if not check_python_version():
        return False
    
    # 检查 pip
    if not check_pip():
        return False
    
    # 需要的包列表
    required_packages = [
        ("requests", "requests"),
        ("DrissionPage", "DrissionPage"),
    ]
    
    # 检查并安装每个包
    all_success = True
    for package, import_name in required_packages:
        if not check_and_install_package(package, import_name):
            all_success = False
    
    # 检查 Chrome
    chrome_ok = check_chrome()
    
    print("=" * 60)
    
    if all_success:
        if chrome_ok:
            print("✅ 所有依赖已就绪")
        else:
            print("⚠️ Python 依赖已安装，但需要安装 Chrome 浏览器")
            print("脚本可能无法正常运行")
    else:
        print("❌ 部分依赖安装失败")
        print("\n请手动安装缺失的依赖:")
        print(f"{sys.executable} -m pip install requests DrissionPage")
    
    return all_success

def run_main_script():
    """运行主脚本"""
    script_path = Path(__file__).parent / "verdent_auto_register.py"
    
    if not script_path.exists():
        print(f"❌ 找不到主脚本: {script_path}")
        return False
    
    print(f"\n运行主脚本: {script_path}")
    print("=" * 60)
    
    # 传递所有命令行参数给主脚本
    try:
        result = subprocess.run(
            [sys.executable, str(script_path)] + sys.argv[1:],
            check=False
        )
        return result.returncode == 0
    except KeyboardInterrupt:
        print("\n用户中断执行")
        return False
    except Exception as e:
        print(f"运行主脚本失败: {e}")
        return False

def main():
    """主函数"""
    print("🚀 Verdent AI 自动注册脚本启动器")
    print(f"系统: {platform.system()} {platform.release()}")
    print(f"Python: {sys.executable}")
    print()
    
    # 检查并安装依赖
    if install_dependencies():
        # 运行主脚本
        success = run_main_script()
        sys.exit(0 if success else 1)
    else:
        print("\n⚠️ 请解决依赖问题后重试")
        sys.exit(1)

if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\n用户中断")
        sys.exit(1)
    except Exception as e:
        print(f"\n未预期的错误: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)
