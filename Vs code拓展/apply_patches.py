#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Verdent VS Code 插件自动化补丁脚本
用于应用防封补丁和无缝切号补丁到 extension.js 文件

使用方法:
    python apply_patches.py [--verify-only] [--restore]

选项:
    --verify-only  仅验证补丁状态，不应用修改
    --restore      从备份文件恢复原始代码
"""

import os
import re
import sys
import shutil
import hashlib
from datetime import datetime
from typing import Tuple, Optional, List, Dict

# 配置
EXTENSION_PATH = os.path.join(os.path.dirname(__file__), "extension", "dist", "extension.js")
BACKUP_SUFFIX = ".backup"
PATCH_MARKER = "__VERDENT_RANDOM_ID__"

# 补丁状态
class PatchStatus:
    NOT_APPLIED = "NOT_APPLIED"
    APPLIED = "APPLIED"
    PARTIAL = "PARTIAL"
    ERROR = "ERROR"


def log_info(msg: str):
    print(f"[INFO] {msg}")


def log_success(msg: str):
    print(f"[✓] {msg}")


def log_warning(msg: str):
    print(f"[!] {msg}")


def log_error(msg: str):
    print(f"[✗] {msg}")


def backup_file(filepath: str) -> str:
    """创建备份文件"""
    backup_path = filepath + BACKUP_SUFFIX
    if not os.path.exists(backup_path):
        shutil.copy2(filepath, backup_path)
        log_info(f"已创建备份: {backup_path}")
    else:
        log_info(f"备份已存在: {backup_path}")
    return backup_path


def restore_from_backup(filepath: str) -> bool:
    """从备份恢复"""
    backup_path = filepath + BACKUP_SUFFIX
    if os.path.exists(backup_path):
        shutil.copy2(backup_path, filepath)
        log_success(f"已从备份恢复: {filepath}")
        return True
    else:
        log_error(f"备份文件不存在: {backup_path}")
        return False


def get_random_id_generator_code() -> str:
    """生成随机ID生成器代码（注入到文件开头）"""
    return '''var __VERDENT_RANDOM_ID__=(function(){var crypto=require('crypto');var id=crypto.randomBytes(32).toString('hex');return id;})();var __VERDENT_GET_RANDOM_ID__=function(){return __VERDENT_RANDOM_ID__;};'''


def check_patch_status(content: str) -> Dict[str, str]:
    """检查各个补丁的应用状态"""
    status = {}
    
    # 检查随机ID生成器
    if PATCH_MARKER in content:
        status["random_id_generator"] = PatchStatus.APPLIED
    else:
        status["random_id_generator"] = PatchStatus.NOT_APPLIED
    
    # 检查 machineIdSync 补丁
    if "return __VERDENT_GET_RANDOM_ID__()" in content and "machineIdSync" in content:
        status["machine_id_sync"] = PatchStatus.APPLIED
    else:
        status["machine_id_sync"] = PatchStatus.NOT_APPLIED
    
    # 检查 loadEnvironment 补丁
    if "[PATCHED]" in content and "deviceId" in content:
        status["load_environment"] = PatchStatus.APPLIED
    else:
        status["load_environment"] = PatchStatus.NOT_APPLIED
    
    # 检查 handleAuthCallback 补丁
    # 检测多种补丁标记
    auth_patched = (
        "Auto-logout for account switching" in content or
        ("handleSignOut" in content and "[Patch]" in content) or
        ("/* [PATCHED] */" in content and "if(false" in content)
    )
    if auth_patched:
        status["auth_callback"] = PatchStatus.APPLIED
    else:
        status["auth_callback"] = PatchStatus.NOT_APPLIED
    
    return status


def apply_random_id_generator(content: str) -> Tuple[str, bool]:
    """应用随机ID生成器补丁"""
    if PATCH_MARKER in content:
        log_info("随机ID生成器已存在，跳过")
        return content, False
    
    # 在 "use strict" 后注入
    pattern = r'^("use strict";)'
    replacement = r'\1' + get_random_id_generator_code()
    
    new_content, count = re.subn(pattern, replacement, content, count=1)
    if count > 0:
        log_success("已注入随机ID生成器")
        return new_content, True
    else:
        log_error("无法找到注入点 ('use strict')")
        return content, False


def apply_machine_id_patch(content: str) -> Tuple[str, bool]:
    """应用 machineIdSync/machineId 补丁"""
    if "return __VERDENT_GET_RANDOM_ID__()" in content:
        log_info("machineId 补丁已存在，跳过")
        return content, False
    
    # 查找 electron-machine-id 模块中的 machineIdSync 和 machineId 函数定义
    # 特征: n.machineIdSync=d,n.machineId=c 其中 d 和 c 是函数
    # 我们需要找到函数 d 的定义并替换其内容
    
    # 模式1: 查找 function d(S){...} 形式的 machineIdSync
    pattern1 = r'(function\s+d\s*\(\s*S\s*\)\s*\{)\s*var\s+D\s*=\s*A\s*\(\s*\(\s*0\s*,\s*h\.execSync\s*\)'
    
    if re.search(pattern1, content):
        # 替换整个函数体
        replacement = r'\1return __VERDENT_GET_RANDOM_ID__();'
        # 需要更精确的替换，找到完整的函数
        pass
    
    # 更通用的方法：查找包含 REG.exe QUERY 的代码块并在其前面添加返回
    # 查找 machineIdSync 函数并修改
    pattern_sync = r'(n\.machineIdSync\s*=\s*)d(\s*,\s*n\.machineId\s*=\s*)c'
    
    if re.search(pattern_sync, content):
        # 在模块定义前添加替换函数
        inject_code = '''var __patched_machineIdSync=function(){return __VERDENT_GET_RANDOM_ID__();};var __patched_machineId=function(){return Promise.resolve(__VERDENT_GET_RANDOM_ID__());};'''
        
        # 替换导出
        new_content = re.sub(
            pattern_sync,
            r'\1__patched_machineIdSync\2__patched_machineId',
            content
        )
        
        # 在 electron-machine-id 模块开始处注入
        # 查找模块定义
        module_pattern = r'(var\s+EFt\s*=\s*fe\s*\(\s*\([^)]+\)\s*=>\s*\{)'
        if re.search(module_pattern, new_content):
            new_content = re.sub(module_pattern, inject_code + r'\1', new_content, count=1)
            log_success("已应用 machineId 补丁 (方法1)")
            return new_content, True
    
    log_warning("machineId 补丁应用失败，尝试备用方法")
    return content, False


def apply_load_environment_patch(content: str) -> Tuple[str, bool]:
    """应用 loadEnvironment 补丁"""
    if "[PATCHED]" in content and "Always use random device ID" in content:
        log_info("loadEnvironment 补丁已存在，跳过")
        return content, False

    # 原始代码结构 (压缩后):
    # async loadEnvironment(){
    #   if(this.get("deviceId")||await this.set("deviceId",await(0,yFt.machineId)(!0)),
    #   !this.get(`projectId__${TAe}`)){
    #     let e=...;await this.set(`projectId__${TAe}`,e)
    #   }
    # }
    #
    # 修改策略:
    # 1. 不使用 return，保留 projectId 设置逻辑
    # 2. 只替换 machineId 调用部分为随机ID

    # 模式1: 替换 await(0,yFt.machineId)(!0) 或类似的 machineId 调用
    pattern1 = r'await\s*\(\s*0\s*,\s*\w+\.machineId\s*\)\s*\(\s*!?\s*0\s*\)'

    if re.search(pattern1, content):
        new_content = re.sub(
            pattern1,
            '/* [PATCHED] Always use random device ID for anti-ban */__VERDENT_GET_RANDOM_ID__()',
            content,
            count=1
        )
        log_success("已应用 loadEnvironment 补丁 (machineId替换)")
        return new_content, True

    # 模式2: 替换整个 this.set("deviceId", ...) 中的值部分
    # 匹配: this.set("deviceId",await(0,yFt.machineId)(!0))
    pattern2 = r'(this\.set\s*\(\s*"deviceId"\s*,\s*)await\s*\([^)]+\.machineId[^)]*\)\s*\([^)]*\)'

    if re.search(pattern2, content):
        new_content = re.sub(
            pattern2,
            r'\1/* [PATCHED] */__VERDENT_GET_RANDOM_ID__()',
            content,
            count=1
        )
        log_success("已应用 loadEnvironment 补丁 (set替换)")
        return new_content, True

    # 备用模式3: 更宽松的匹配 - 直接在 deviceId 设置处替换
    pattern3 = r'(this\.get\s*\(\s*"deviceId"\s*\)\s*\|\|\s*await\s+this\.set\s*\(\s*"deviceId"\s*,\s*)([^)]+)(\))'

    if re.search(pattern3, content):
        new_content = re.sub(
            pattern3,
            r'\1/* [PATCHED] */__VERDENT_GET_RANDOM_ID__()\3',
            content,
            count=1
        )
        log_success("已应用 loadEnvironment 补丁 (备用方法)")
        return new_content, True

    log_warning("loadEnvironment 补丁应用失败")
    return content, False


def apply_auth_callback_patch(content: str) -> Tuple[str, bool]:
    """应用 handleAuthCallback 无缝切号补丁"""
    if "Auto-logout for account switching" in content:
        log_info("handleAuthCallback 补丁已存在，跳过")
        return content, False

    # 原始代码结构:
    # async handleAuthCallback(e,i){
    #   let n=Date.now(),a=await Nr(this.context,"userInfo")||{};
    #   if(this.isAuth||a.isLogin){
    #     this.callbackLogStatus(...);...;return
    #   }
    #   this.isAuth=!0;
    #   // ... 后续登录逻辑
    # }
    #
    # 修改策略:
    # 1. 在 if(this.isAuth||a.isLogin) 之前插入自动登出逻辑
    # 2. 将原有的条件检查改为 if(false) 使其永不执行（保留原代码结构避免语法错误）

    # 模式1: 精确匹配并在条件检查前插入登出逻辑
    # 查找: if(this.isAuth||a.isLogin){
    pattern1 = r'(if\s*\(\s*this\.isAuth\s*\|\|\s*a\.isLogin\s*\)\s*\{)'

    if re.search(pattern1, content):
        # 在原 if 语句前插入自动登出逻辑
        replacement = '/* [PATCHED] Auto-logout for account switching */if(a.isLogin){await this.handleSignOut();}if(false){'
        new_content = re.sub(pattern1, replacement, content, count=1)
        if new_content != content:
            log_success("已应用 handleAuthCallback 补丁")
            return new_content, True

    log_warning("handleAuthCallback 补丁应用失败")
    return content, False


def apply_all_patches(content: str) -> Tuple[str, Dict[str, bool]]:
    """应用所有补丁"""
    results = {}

    # 1. 随机ID生成器（必须首先应用）
    content, applied = apply_random_id_generator(content)
    results["random_id_generator"] = applied

    # 2. machineId 补丁
    content, applied = apply_machine_id_patch(content)
    results["machine_id"] = applied

    # 3. loadEnvironment 补丁
    content, applied = apply_load_environment_patch(content)
    results["load_environment"] = applied

    # 4. handleAuthCallback 补丁
    content, applied = apply_auth_callback_patch(content)
    results["auth_callback"] = applied

    return content, results


def verify_patches(filepath: str) -> Dict[str, str]:
    """验证补丁状态"""
    try:
        with open(filepath, 'r', encoding='utf-8') as f:
            content = f.read()
        return check_patch_status(content)
    except Exception as e:
        log_error(f"读取文件失败: {e}")
        return {}


def main():
    """主函数"""
    print("=" * 60)
    print("Verdent VS Code 插件自动化补丁工具")
    print("=" * 60)
    print()

    # 解析命令行参数
    verify_only = "--verify-only" in sys.argv
    restore = "--restore" in sys.argv

    # 检查文件是否存在
    if not os.path.exists(EXTENSION_PATH):
        log_error(f"找不到 extension.js 文件: {EXTENSION_PATH}")
        log_info("请确保脚本位于正确的目录中")
        return 1

    log_info(f"目标文件: {EXTENSION_PATH}")

    # 恢复模式
    if restore:
        print("\n[恢复模式]")
        if restore_from_backup(EXTENSION_PATH):
            return 0
        return 1

    # 验证模式
    if verify_only:
        print("\n[验证模式]")
        status = verify_patches(EXTENSION_PATH)
        print("\n补丁状态:")
        print("-" * 40)
        for patch_name, patch_status in status.items():
            icon = "✓" if patch_status == PatchStatus.APPLIED else "✗"
            print(f"  [{icon}] {patch_name}: {patch_status}")
        return 0

    # 应用补丁模式
    print("\n[应用补丁模式]")

    # 创建备份
    backup_file(EXTENSION_PATH)

    # 读取文件
    try:
        with open(EXTENSION_PATH, 'r', encoding='utf-8') as f:
            content = f.read()
        log_info(f"文件大小: {len(content)} 字符")
    except Exception as e:
        log_error(f"读取文件失败: {e}")
        return 1

    # 应用补丁
    print("\n正在应用补丁...")
    print("-" * 40)
    new_content, results = apply_all_patches(content)

    # 检查是否有修改
    if new_content == content:
        log_info("没有需要应用的补丁（可能已全部应用）")
    else:
        # 写入文件
        try:
            with open(EXTENSION_PATH, 'w', encoding='utf-8') as f:
                f.write(new_content)
            log_success("补丁已写入文件")
        except Exception as e:
            log_error(f"写入文件失败: {e}")
            return 1

    # 验证结果
    print("\n验证补丁状态...")
    print("-" * 40)
    status = verify_patches(EXTENSION_PATH)

    all_applied = True
    for patch_name, patch_status in status.items():
        icon = "✓" if patch_status == PatchStatus.APPLIED else "✗"
        print(f"  [{icon}] {patch_name}: {patch_status}")
        if patch_status != PatchStatus.APPLIED:
            all_applied = False

    print()
    if all_applied:
        log_success("所有补丁已成功应用！")
    else:
        log_warning("部分补丁未能应用，请检查日志")

    print("\n" + "=" * 60)
    print("补丁应用完成")
    print("=" * 60)

    return 0 if all_applied else 1


if __name__ == "__main__":
    sys.exit(main())

