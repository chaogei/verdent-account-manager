#!/usr/bin/env python
"""
构建 verdent_auto_register.exe 的脚本
使用 PyInstaller 打包，确保所有依赖都被正确包含
"""

import subprocess
import sys
import os
from pathlib import Path

def build_executable():
    """构建可执行文件"""
    
    # 确保在正确的目录
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    print("开始构建 verdent_auto_register.exe...")
    
    # PyInstaller 命令
    cmd = [
        sys.executable, "-m", "PyInstaller",
        "--onefile",                    # 打包成单个文件
        "--console",                     # 控制台程序
        "--clean",                       # 清理临时文件
        "--noconfirm",                   # 覆盖输出目录
        "--collect-all", "DrissionPage", # 收集 DrissionPage 的所有内容
        "--collect-all", "websocket",    # 收集 websocket 的所有内容
        "--collect-all", "lxml",         # 收集 lxml 的所有内容
        "--collect-all", "tldextract",   # 收集 tldextract 的所有内容
        "--hidden-import", "DrissionPage._base.chromium",
        "--hidden-import", "DrissionPage._pages.chromium_page",
        "--hidden-import", "DrissionPage._functions.browser",
        "--hidden-import", "DrissionPage._functions.elements",
        "--hidden-import", "DrissionPage._units.setter",
        "--hidden-import", "DrissionPage._units.waiter",
        "--hidden-import", "websocket._core",
        "--hidden-import", "websocket._app",
        "--hidden-import", "lxml.html",
        "--hidden-import", "lxml.etree",
        "--hidden-import", "cssselect",
        "--hidden-import", "urllib3",
        "--hidden-import", "requests",
        "--hidden-import", "certifi",
        "--hidden-import", "charset_normalizer",
        "--hidden-import", "idna",
        "--name", "verdent_auto_register",
        "verdent_auto_register.py"
    ]
    
    # 如果存在图标文件，添加图标
    icon_path = Path("Verdent_account_manger/src-tauri/icons/icon.ico")
    if icon_path.exists():
        cmd.extend(["--icon", str(icon_path)])
    
    # 执行打包命令
    try:
        result = subprocess.run(cmd, check=True, capture_output=True, text=True)
        print("打包成功！")
        print(result.stdout)
        
        # 检查输出文件
        exe_path = Path("dist/verdent_auto_register.exe")
        if exe_path.exists():
            size_mb = exe_path.stat().st_size / (1024 * 1024)
            print(f"生成的文件: {exe_path}")
            print(f"文件大小: {size_mb:.2f} MB")
            
            # 复制到 resources 目录
            resources_dir = Path("Verdent_account_manger/resources")
            resources_dir.mkdir(exist_ok=True)
            
            import shutil
            target_path = resources_dir / "verdent_auto_register.exe"
            shutil.copy2(exe_path, target_path)
            print(f"已复制到: {target_path}")
            
            return True
        else:
            print("错误：未找到生成的 exe 文件")
            return False
            
    except subprocess.CalledProcessError as e:
        print(f"打包失败: {e}")
        print(f"错误输出: {e.stderr}")
        return False
    except Exception as e:
        print(f"发生错误: {e}")
        return False

if __name__ == "__main__":
    success = build_executable()
    sys.exit(0 if success else 1)
