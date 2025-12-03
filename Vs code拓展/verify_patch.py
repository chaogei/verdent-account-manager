#!/usr/bin/env python3
"""
Verdent 插件防封补丁验证脚本
用于检查 extension.js 是否已正确应用补丁
使用分块读取以提高大文件处理效率
"""

import sys
from pathlib import Path

def verify_patch(extension_path: str = None) -> bool:
    """验证补丁是否正确应用（使用关键字搜索而非正则）"""

    if extension_path is None:
        extension_path = Path(__file__).parent / "extension" / "dist" / "extension.js"
    else:
        extension_path = Path(extension_path)

    if not extension_path.exists():
        print(f"❌ 文件不存在: {extension_path}")
        return False

    print(f"📁 检查文件: {extension_path}")
    print("-" * 60)

    # 使用关键字列表进行快速检查
    checks = {
        "随机ID生成器注入": "__VERDENT_RANDOM_ID__",
        "machineIdSync 修改": "[PATCHED] Return random ID instead of real machine ID",
        "loadEnvironment 修改": "[PATCHED] Always use random device ID for anti-ban",
        "无缝切号功能": "[PATCHED] Auto-logout before new login for seamless account switching",
    }

    results = {name: False for name in checks}

    # 分块读取文件
    with open(extension_path, 'r', encoding='utf-8') as f:
        for line in f:
            for name, keyword in checks.items():
                if keyword in line:
                    results[name] = True

    all_passed = True
    for name, passed in results.items():
        if passed:
            print(f"✅ {name}: 已应用")
        else:
            print(f"❌ {name}: 未检测到")
            all_passed = False

    print("-" * 60)

    if all_passed:
        print("\n🎉 补丁验证通过！所有修改已正确应用。")
    else:
        print("\n⚠️  补丁验证失败！部分修改未检测到。")

    return all_passed

if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else None
    success = verify_patch(path)
    sys.exit(0 if success else 1)

