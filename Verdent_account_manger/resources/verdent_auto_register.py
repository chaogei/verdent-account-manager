#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Verdent AI 自动注册脚本
使用 DrissionPage 自动化 + tempmail.plus 免费 API (无需 API Key)
"""

import sys
import io
import time
import tempfile
import os
import traceback
import random
import json
import string
import shutil
import requests
import uuid
from datetime import datetime, timedelta
from typing import Optional, Dict, Any
from urllib.parse import quote
from DrissionPage import ChromiumPage, ChromiumOptions
from concurrent.futures import ThreadPoolExecutor, as_completed
import re

# 强制使用 UTF-8 编码输出,解决 Windows GBK 编码问题
if sys.platform == 'win32':
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding='utf-8', errors='replace')
    except Exception:
        pass  # 如果设置失败,继续使用默认编码


class TempMailPlusAPI:
    DOMAINS = [
        "mailto.plus",
        "fexpost.com",
        "fexbox.org",
        "mailbox.in.ua",
        "rover.info",
        "chitthi.in",
        "fextemp.com",
        "any.pink",
        "merepost.com"
    ]
    
    def __init__(self):
        self.base_url = "https://tempmail.plus/api"
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36",
            "Accept": "application/json, text/javascript, */*; q=0.01",
            "Accept-Language": "zh-CN,zh;q=0.9",
            "Referer": "https://tempmail.plus/zh/",
            "X-Requested-With": "XMLHttpRequest",
            "sec-ch-ua": '"Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99"',
            "sec-ch-ua-mobile": "?0",
            "sec-ch-ua-platform": '"Windows"',
        })
    
    def _generate_random_name(self) -> str:
        first_names = [
            "lisa", "jake", "emma", "noah", "olivia", "liam", "ava", "ethan",
            "sophia", "mason", "mia", "william", "charlotte", "james", "amelia",
            "benjamin", "harper", "lucas", "evelyn", "henry", "abigail", "alex",
            "emily", "michael", "ella", "daniel", "madison", "david", "scarlett"
        ]
        
        last_names = [
            "smith", "johnson", "williams", "brown", "jones", "garcia", "miller",
            "davis", "rodriguez", "martinez", "anderson", "taylor", "thomas",
            "jackson", "white", "harris", "clark", "lewis", "walker", "hall"
        ]
        
        first = random.choice(first_names).capitalize()
        last = random.choice(last_names).capitalize()
        
        numbers = ''.join(random.choices(string.digits, k=random.randint(3, 6)))
        
        separator = random.choice(['_', '.', ''])
        
        username = f"{first}{separator}{last}{numbers}"
        
        return username.lower()
    
    def create_mailbox(self) -> Optional[Dict[str, Any]]:
        try:
            domain = random.choice(self.DOMAINS)
            username = self._generate_random_name()
            email = f"{username}@{domain}"
            
            self.session.cookies.set('email', quote(email))

            print(f"[OK] 生成临时邮箱: {email}")

            return {
                "email": email,
                "domain": domain
            }
        except Exception as e:
            print(f"[X] 创建邮箱失败: {e}")
            return None
    
    def get_verification_code(self, email: str, max_retries: int = 60, initial_interval: float = 1.0) -> Optional[str]:
        """
        获取验证码,使用渐进式轮询策略

        优化策略:
        - 前 10 次: 每 1 秒检查一次 (邮件通常在 5-15 秒内到达)
        - 第 11-30 次: 每 1.5 秒检查一次
        - 第 31 次以后: 每 2 秒检查一次

        总最大等待时间: 约 90 秒 (10×1 + 20×1.5 + 30×2 = 100 秒)
        """
        encoded_email = quote(email)

        for i in range(max_retries):
            try:
                response = self.session.get(
                    f"{self.base_url}/mails",
                    params={
                        "email": email,
                        "first_id": "0",
                        "epin": ""
                    },
                    timeout=10
                )

                if response.status_code == 200:
                    data = response.json()

                    if data.get("result") and data.get("mail_list"):
                        mail_list = data.get("mail_list", [])

                        for mail in mail_list:
                            if "verdent" in mail.get("from_mail", "").lower():
                                mail_id = mail.get("mail_id")

                                detail_response = self.session.get(
                                    f"{self.base_url}/mails/{mail_id}",
                                    params={
                                        "email": email,
                                        "epin": ""
                                    },
                                    timeout=10
                                )

                                if detail_response.status_code == 200:
                                    detail_data = detail_response.json()

                                    if detail_data.get("result"):
                                        text_content = detail_data.get("text", "")
                                        html_content = detail_data.get("html", "")

                                        code_pattern = r'\b\d{6}\b'

                                        match = re.search(code_pattern, text_content)
                                        if not match:
                                            match = re.search(code_pattern, html_content)

                                        if match:
                                            code = match.group(0)
                                            print(f"[OK] 验证码获取成功: {code} (耗时 {i * initial_interval:.1f} 秒)")
                                            return code

                # 渐进式轮询间隔
                if i < 10:
                    interval = 1.0  # 前 10 次: 1 秒
                elif i < 30:
                    interval = 1.5  # 第 11-30 次: 1.5 秒
                else:
                    interval = 2.0  # 第 31 次以后: 2 秒

                print(f"[*] 等待验证码邮件 ({i+1}/{max_retries})...", flush=True)
                time.sleep(interval)

            except Exception as e:
                print(f"[!] 获取邮件失败: {e}", flush=True)
                # 出错时使用较短的间隔快速重试
                time.sleep(1.0)

        return None


class VerdentAutoRegister:
    def __init__(self, headless: bool = False):
        self.tempmail = TempMailPlusAPI()
        self.headless = headless
        self.signup_url = "https://www.verdent.ai/signup?source=dashboard"

        self.xpath_email = '/html/body/div/div[1]/div[2]/form/div[1]/div/input'
        self.xpath_send_code = '/html/body/div/div[1]/div[2]/form/div[1]/div/button'
        self.xpath_code_input = '/html/body/div/div[1]/div[2]/form/div[2]/div/input'
        self.xpath_password = '/html/body/div/div[1]/div[2]/form/div[3]/div/input'
        # Cloudflare Turnstile 验证码容器在验证前后位置会变化
        self.xpath_captcha_before_verify = '/html/body/div/div[1]/div[2]/form/div[5]/div'  # 验证前位置
        self.xpath_captcha_after_verify = '/html/body/div/div[1]/div[2]/form/div[4]/div'   # 验证后位置
        self.xpath_register_btn = '/html/body/div/div[1]/div[2]/div[3]'

    @staticmethod
    def generate_random_password(length: int = 14) -> str:
        """
        生成符合 Verdent AI 要求的随机密码

        密码要求:
        - 长度: 12-16 个字符
        - 包含大写字母 (A-Z)
        - 包含小写字母 (a-z)
        - 包含数字 (0-9)
        - 包含特殊字符 (@, #, $, %, &, *)

        Args:
            length: 密码长度 (默认 14)

        Returns:
            生成的随机密码
        """
        if length < 12:
            length = 12
        elif length > 16:
            length = 16

        # 定义字符集
        uppercase = string.ascii_uppercase  # A-Z
        lowercase = string.ascii_lowercase  # a-z
        digits = string.digits              # 0-9
        special = '@#$%&*'                  # 特殊字符

        # 确保至少包含每种类型的字符
        password_chars = [
            random.choice(uppercase),  # 至少 1 个大写字母
            random.choice(lowercase),  # 至少 1 个小写字母
            random.choice(digits),     # 至少 1 个数字
            random.choice(special),    # 至少 1 个特殊字符
        ]

        # 填充剩余长度
        all_chars = uppercase + lowercase + digits + special
        remaining_length = length - len(password_chars)
        password_chars.extend(random.choice(all_chars) for _ in range(remaining_length))

        # 打乱顺序
        random.shuffle(password_chars)

        return ''.join(password_chars)
    
    def _create_browser(self) -> ChromiumPage:

        
        options = ChromiumOptions()
        # 设置浏览器启动参数
        # 使用随机端口避免冲突
        port = random.randint(9200, 9999)
        options.set_local_port(port)
        
        # 创建临时用户数据目录，避免与已有Chrome实例冲突
        temp_dir = tempfile.mkdtemp(prefix='verdent_chrome_')
        options.set_user_data_path(temp_dir)
        
        if self.headless:
            options.headless(True)
            options.set_argument('--headless=new')  # 使用新的headless模式
        
        #无痕模式
        options.set_argument('--incognito')
        options.set_argument('--disable-gpu')
        options.set_argument('--no-sandbox')  # Linux兼容性
        options.set_argument('--disable-dev-shm-usage')  # 解决共享内存问题
        
        # Windows系统特殊处理
        if os.name == 'nt':
            options.set_argument('--disable-gpu-sandbox')
        
        try:
            # 尝试创建浏览器页面
            print(f"[*] 创建浏览器，临时目录: {temp_dir}", flush=True)
            page = ChromiumPage(addr_or_opts=options)
            # 保存临时目录路径，以便后续清理
            page._temp_user_data_dir = temp_dir
            print("[✓] 浏览器创建成功", flush=True)
            return page
        except ValueError as ve:
            # 处理地址解析错误
            if "not enough values to unpack" in str(ve):
                print("[!] 地址解析错误，尝试不指定选项创建浏览器...", flush=True)
                try:
                    # 尝试使用默认配置
                    page = ChromiumPage()
                    page._temp_user_data_dir = temp_dir
                    print("[✓] 使用默认配置创建浏览器成功", flush=True)
                    return page
                except Exception as e2:
                    print(f"[X] 默认配置也失败: {e2}", flush=True)
                    raise e2
            else:
                raise ve
        except Exception as e:
            # 如果创建失败，清理临时目录
            print(f"[X] 创建浏览器失败: {e}", flush=True)
            
            if os.path.exists(temp_dir):
                try:
                    shutil.rmtree(temp_dir, ignore_errors=True)
                except:
                    pass
            raise e
    
    def _check_browser_alive(self, page: ChromiumPage) -> bool:
        """
        检查浏览器是否仍然连接
        返回: True=连接正常, False=浏览器已关闭
        """
        try:
            # 尝试获取当前URL,如果浏览器已关闭会抛出异常
            _ = page.url
            return True
        except Exception as e:
            print(f"[X] 浏览器连接已断开: {e}", flush=True)
            return False
    
    def _ensure_browser_alive(self, page: ChromiumPage):
        """
        确保浏览器仍然连接,如果断开则抛出异常
        """
        if not self._check_browser_alive(page):
            raise RuntimeError("浏览器已被用户关闭或连接中断，注册流程终止")
    
    def check_turnstile_success(self, wait_time=0.5, after_click=False):
        """
        检查 Cloudflare Turnstile 是否已经自动通过
        检测 Shadow DOM 内的成功状态元素

        注意: 验证成功后,验证码容器会从 div[5] 移动到 div[4]

        Args:
            wait_time: 单次查找元素的超时时间(秒),默认0.5秒
                      优化: 使用更短的超时时间以加快检测速度
            after_click: 是否在点击验证框之后调用
                        True: 只检查验证后的位置 (div[4]) - 更快
                        False: 检查两个位置 (用于初始检测)

        Returns:
            是否已经成功
        """
        try:
            # 根据调用时机选择检查策略
            if after_click:
                # 点击后只检查验证后的位置 (div[4]),避免浪费时间
                xpaths_to_check = [self.xpath_captcha_after_verify]
            else:
                # 初始检测时尝试两个位置
                # 优先检查验证后的位置,因为成功后通常在这里
                xpaths_to_check = [self.xpath_captcha_before_verify]

            for xpath in xpaths_to_check:
                try:
                    success_elem = (
                        self.page.ele(f"xpath:{xpath}", timeout=wait_time)
                        .child()
                        .shadow_root.ele("tag:iframe", timeout=wait_time)
                        .ele("tag:body", timeout=wait_time)
                        .sr("@id=success")
                    )

                    if success_elem:
                        style = success_elem.attr("style") or ""
                        if "visibility: visible" in style or "display: grid" in style:
                            success_text = success_elem.ele("@id=success-text", timeout=wait_time)
                            if success_text:
                                text_content = success_text.text
                                if text_content and ("Success" in text_content or "成功" in text_content):
                                    return True
                except:
                    continue

            return False
        except Exception:
            return False
    
    def handle_verification_manual(self) -> bool:
        """
        手动等待用户完成验证
        注意: 实际上验证可能已经在自动进行,只是需要更多时间
        :return: 是否成功
        """
        print("\n" + "="*70, flush=True)
        print("[WARN] 需要手动完成验证", flush=True)
        print("="*70, flush=True)
        print("请在浏览器中手动完成验证码验证", flush=True)
        print("(如果验证框已经在自动验证中,请耐心等待)", flush=True)
        print("验证完成后,脚本将自动继续...", flush=True)
        print("等待中...\n", flush=True)

        max_wait = 60  # 最大等待 60 秒
        check_interval = 0.5  # 每 0.5 秒检查一次
        elapsed = 0
        last_progress_time = 0

        while elapsed < max_wait:
            if self.check_turnstile_success(wait_time=0.3):
                print(f"\n[OK] 检测到验证已完成! (耗时 {elapsed:.1f} 秒)", flush=True)
                return True

            # 每 5 秒显示一次进度
            if elapsed - last_progress_time >= 5:
                remaining = max_wait - elapsed
                print(f"   [*] 等待手动验证... (已等待 {elapsed:.1f}s / 最多 {max_wait}s, 剩余 {remaining:.1f}s)", flush=True)
                last_progress_time = elapsed

            time.sleep(check_interval)
            elapsed += check_interval

        print(f"\n[!] 等待超时 ({max_wait} 秒),验证可能未完成", flush=True)
        return False
    
    def handle_cloudflare_turnstile(self):
        """
        处理 Cloudflare Turnstile 验证码
        直接点击验证框触发验证

        注意: 点击验证框时使用验证前的位置(div[5])

        :return: 是否成功
        """
        try:
            print("\n[CAPTCHA] 检测到 Cloudflare Turnstile 验证码,尝试自动处理...")

            try:
                print("   尝试点击验证框...")
                # 使用验证前的位置(div[5])来定位验证框
                captcha_container = self.page.ele(f"xpath:{self.xpath_captcha_before_verify}", timeout=5)
                
                if captcha_container:
                    challenge_check = (
                        captcha_container
                        .child()
                        .shadow_root.ele("tag:iframe", timeout=3)
                        .ele("tag:body", timeout=3)
                        .sr("tag:input")
                    )
                    
                    if challenge_check:
                        challenge_check.click()
                        print("[OK] 已点击验证框,等待验证...", flush=True)

                        # # 智能轮询检测验证是否完成
                        # max_wait = 30  # 最大等待 30 秒
                        # check_interval = 0.5  # 每 0.5 秒检查一次
                        # elapsed = 0
                        # last_progress_time = 0

                        # while elapsed < max_wait:
                        #     # 检查验证是否成功
                        #     # 优化: 点击后只检查验证后的位置 (div[4]),避免浪费时间
                        #     if self.check_turnstile_success(wait_time=0.3, after_click=True):
                        #         print(f"[OK] 验证码已自动通过! (耗时 {elapsed:.1f} 秒)", flush=True)
                        #         return True

                        #     # 每 2 秒显示一次进度
                        #     if elapsed - last_progress_time >= 2:
                        #         remaining = max_wait - elapsed
                        #         print(f"   [*] 等待验证中... (已等待 {elapsed:.1f}s / 最多 {max_wait}s, 剩余 {remaining:.1f}s)", flush=True)
                        #         last_progress_time = elapsed

                        #     time.sleep(check_interval)
                        #     elapsed += check_interval

                        # print(f"[!] 验证码未在 {max_wait} 秒内完成,可能需要手动处理", flush=True)
            except Exception as e:
                print(f"⚠️  点击验证框失败: {e}")

            print("[WARN] 切换到手动模式...")
            return self.handle_verification_manual()

        except Exception as e:
            print(f"[ERROR] 处理验证码时出错: {e}")
            return self.handle_verification_manual()
    
    def _solve_captcha(self, page: ChromiumPage) -> bool:
        self.page = page
        return self.handle_cloudflare_turnstile()
    
    def register_account(self, password: str = "VerdentAI@2024", use_random_password: bool = False) -> Optional[Dict[str, Any]]:
        page = None
        email = None

        # 如果启用随机密码,生成新密码
        if use_random_password:
            password = self.generate_random_password()
            print(f"[*] 已生成随机密码: {password}", flush=True)

        try:
            print("\n" + "="*70, flush=True)
            print("[START] 开始自动注册 Verdent AI 账号", flush=True)
            print("="*70, flush=True)

            print("[*] 步骤 1: 生成临时邮箱...", flush=True)
            mailbox = self.tempmail.create_mailbox()
            if not mailbox:
                print("[X] 生成临时邮箱失败", flush=True)
                return None

            email = mailbox["email"]
            print(f"[OK] 临时邮箱: {email}", flush=True)

            print("[*] 步骤 2: 启动浏览器并打开注册页面...", flush=True)
            page = self._create_browser()
            page.get(self.signup_url)
            # 优化: 移除固定等待,直接查找元素时会自动等待
            # time.sleep(2)  # 已移除

            print("[*] 步骤 3: 填写邮箱地址...", flush=True)
            self._ensure_browser_alive(page)  # 检查浏览器连接
            email_input = page.ele(f"xpath:{self.xpath_email}", timeout=10)
            if not email_input:
                print("[X] 未找到邮箱输入框", flush=True)
                return None
            email_input.input(email)
            # 优化: 缩短等待时间,输入后立即继续
            time.sleep(0.1)

            print("[*] 步骤 4: 发送验证码...", flush=True)
            # 使用API发送验证码，而不是点击按钮
            send_code_url = "https://login.verdent.ai/passport/sendVerifyCode"
            headers = {
                "accept": "application/json, text/plain, */*",
                "accept-language": "zh-CN,zh;q=0.9",
                "cache-control": "no-cache",
                "content-type": "application/json",
                "pragma": "no-cache",
                "priority": "u=1, i",
                "sec-ch-ua": '"Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99"',
                "sec-ch-ua-mobile": "?0",
                "sec-ch-ua-platform": '"Windows"',
                "sec-fetch-dest": "empty",
                "sec-fetch-mode": "cors",
                "sec-fetch-site": "same-site",
                "Referer": "https://www.verdent.ai/"
            }
            
            # 获取当前页面的cookies
            cookies_dict = {}
            try:
                cookies = page.cookies(as_dict=False)
                for cookie in cookies:
                    cookies_dict[cookie['name']] = cookie['value']
            except:
                pass
            
            # 发送验证码请求
            try:
                response = requests.post(
                    send_code_url,
                    json={"email": email},
                    headers=headers,
                    cookies=cookies_dict,
                    timeout=10
                )
                
                if response.status_code == 200:
                    result = response.json()
                    if result.get("errCode") == 0:
                        print("[OK] 验证码发送成功", flush=True)
                    else:
                        print(f"[X] 验证码发送失败: {result.get('errMsg', '未知错误')}", flush=True)
                        return None
                else:
                    print(f"[X] 验证码发送失败，HTTP状态码: {response.status_code}", flush=True)
                    return None
            except Exception as e:
                print(f"[X] 发送验证码请求失败: {e}", flush=True)
                return None

            print("[*] 步骤 5: 等待并获取验证码...", flush=True)
            verification_code = self.tempmail.get_verification_code(email)
            if not verification_code:
                print("[X] 未收到验证码", flush=True)
                return None
            print(f"[OK] 验证码: {verification_code}", flush=True)

            print("[*] 步骤 6: 输入验证码...", flush=True)
            self._ensure_browser_alive(page)  # 检查浏览器连接
            code_input = page.ele(f"xpath:{self.xpath_code_input}", timeout=5)
            if code_input:
                code_input.input(verification_code)
                # 优化: 缩短等待时间
                time.sleep(0.1)
            else:
                print("[X] 未找到验证码输入框", flush=True)
                return None

            print("[*] 步骤 7: 输入密码...", flush=True)
            self._ensure_browser_alive(page)  # 检查浏览器连接
            if use_random_password:
                print(f"    使用随机生成的密码: {password}", flush=True)
            password_input = page.ele(f"xpath:{self.xpath_password}", timeout=5)
            if password_input:
                password_input.input(password)
                print("[OK] 密码已输入", flush=True)
                # 优化: 缩短等待时间
                time.sleep(0.1)
            else:
                print("[X] 未找到密码输入框", flush=True)
                return None

            print("[*] 步骤 8: 处理人机验证...", flush=True)
            self._ensure_browser_alive(page)  # 检查浏览器连接
            self._solve_captcha(page)
            # 优化: 缩短等待时间,验证完成后立即继续
            time.sleep(0.1)

            print("[*] 步骤 9: 点击注册按钮...", flush=True)
            self._ensure_browser_alive(page)  # 检查浏览器连接
            register_btn = page.ele(f"xpath:{self.xpath_register_btn}", timeout=5)
            if register_btn:
                register_btn.click()
                print("[OK] 已点击注册按钮", flush=True)
            else:
                print("[X] 未找到注册按钮", flush=True)
                return None

            print("[*] 步骤 10: 等待注册完成...", flush=True)

            # 智能等待页面跳转到 dashboard
            max_wait = 15  # 最大等待 15 秒
            check_interval = 0.3  # 优化: 缩短检查间隔至 0.3 秒,更快检测到跳转
            elapsed = 0
            last_progress_time = 0

            while elapsed < max_wait:
                # 检查浏览器连接
                self._ensure_browser_alive(page)
                
                current_url = page.url

                # 检查是否已跳转到 dashboard (检查 URL 路径,忽略参数)
                # 正确的 dashboard URL: https://www.verdent.ai/dashboard?source=signup&login_from=signup
                # 错误的 signup URL: https://www.verdent.ai/signup?source=dashboard
                if "/dashboard" in current_url and "/signup" not in current_url:
                    print(f"[OK] 注册成功! (耗时 {elapsed:.1f} 秒)", flush=True)
                    break

                # 每 2 秒显示一次进度
                if elapsed - last_progress_time >= 2:
                    print(f"   [*] 等待页面跳转... (已等待 {elapsed:.1f}s / 最多 {max_wait}s, 当前URL: {current_url[:50]}...)", flush=True)
                    last_progress_time = elapsed

                time.sleep(check_interval)
                elapsed += check_interval
            else:
                # 超时后再检查一次
                current_url = page.url
                if "/dashboard" not in current_url or "/signup" in current_url:
                    print(f"[!] 注册可能失败,当前 URL: {current_url}", flush=True)
                    return None

            # 确认注册成功
            current_url = page.url
            if "/dashboard" in current_url and "/signup" not in current_url:
                print("[OK] 确认注册成功!", flush=True)

                # 步骤 11: 提取 token Cookie
                print("[*] 步骤 11: 提取 token Cookie...", flush=True)
                token = None

                try:
                    # 等待页面完全加载
                    print("[*] 等待 dashboard 页面完全加载...", flush=True)
                    # 优化: 缩短等待时间至 1.5 秒,通常足够 Cookie 设置完成
                    time.sleep(1.5)

                    print(f"[*] 当前 URL: {page.url}", flush=True)

                    # 从浏览器 Cookies 中提取 token
                    print("[*] 从浏览器 Cookies 中提取 token...", flush=True)
                    try:
                        # DrissionPage 获取 cookies: cookies(all_info=True) 返回 CookiesList 对象
                        all_cookies = page.cookies(all_info=True)
                        print(f"[*] 获取到 {len(all_cookies) if all_cookies else 0} 个 cookies", flush=True)

                        if all_cookies:
                            # 遍历所有 cookies 查找 token
                            for cookie in all_cookies:
                                if isinstance(cookie, dict) and cookie.get('name') == 'token':
                                    token = cookie.get('value')
                                    print(f"[OK] 提取 Token 成功 (前50字符): {token[:50]}...", flush=True)
                                    break
                    except Exception as e:
                        print(f"[!] Cookies 获取失败: {e}", flush=True)

                    # 如果未找到 token,尝试通过密码登录 API 获取
                    if not token:
                        print("[*] Cookie 中未找到 token,尝试通过登录 API 获取...", flush=True)
                        try:
                            login_url = "https://login.verdent.ai/passport/login"
                            login_payload = {
                                "email": email,
                                "password": password
                            }
                            login_headers = {
                                "Accept": "application/json, text/plain, */*",
                                "Content-Type": "application/json",
                                "Origin": "https://www.verdent.ai",
                                "Referer": "https://www.verdent.ai/",
                                "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
                            }

                            response = requests.post(login_url, json=login_payload, headers=login_headers, timeout=30)

                            if response.status_code == 200:
                                result = response.json()
                                if result.get('errCode') == 0 and result.get('data'):
                                    token = result['data'].get('token')
                                    if token:
                                        print(f"[OK] 通过登录 API 获取 Token 成功 (前50字符): {token[:50]}...", flush=True)
                                    else:
                                        print(f"[!] 登录成功但响应中没有 token 字段", flush=True)
                                else:
                                    print(f"[!] 登录失败: {result.get('errMsg', '未知错误')}", flush=True)
                            else:
                                print(f"[!] 登录请求失败,状态码: {response.status_code}", flush=True)
                        except Exception as e:
                            print(f"[!] 登录 API 调用失败: {e}", flush=True)

                    if not token:
                        print("[!] 警告: 未能获取到 token", flush=True)
                        print("[!] 建议: 账户已创建,可稍后手动登录获取 Token", flush=True)

                except Exception as e:
                    print(f"[ERROR] 提取 token 时出错: {e}", flush=True)
                    
                    traceback.print_exc()
                    token = None

                # 步骤 12: 获取额度信息
                print("[*] 步骤 12: 获取额度信息...", flush=True)
                quota_remaining = None
                quota_used = None
                quota_total = None
                subscription_type = None
                trial_days = None
                register_time = datetime.now()  # 默认使用当前时间
                expire_time = None
                
                if token:
                    try:
                        # 调用dashboard API获取额度信息
                        dashboard_url = "https://www.verdent.ai/dashboard/__data.json"
                        params = {
                            "source": "signup",
                            "login_from": "signup",
                            "x-sveltekit-invalidated": "01"
                        }
                        headers = {
                            "accept": "*/*",
                            "accept-language": "zh-CN,zh;q=0.9",
                            "cache-control": "no-cache",
                            "pragma": "no-cache",
                            "sec-fetch-dest": "empty",
                            "sec-fetch-mode": "cors",
                            "sec-fetch-site": "same-origin",
                            "Referer": "https://www.verdent.ai/signup?source=dashboard"
                        }
                        
                        # 使用token作为cookie
                        cookies = {"token": token}
                        
                        response = requests.get(
                            dashboard_url,
                            params=params,
                            headers=headers,
                            cookies=cookies,
                            timeout=15
                        )
                        
                        if response.status_code == 200:
                            data = response.json()
                            # 解析SvelteKit的扁平化数据结构
                            if data.get("type") == "data" and data.get("nodes"):
                                nodes = data["nodes"]
                                if len(nodes) > 1 and nodes[1].get("type") == "data":
                                    data_array = nodes[1].get("data", [])
                                    
                                    # 查找account_info对象
                                    for item in data_array:
                                        if isinstance(item, dict) and "available_sub_credits" in item:
                                            # 获取索引值
                                            total_idx = item.get("available_sub_credits")
                                            used_idx = item.get("used_sub_credits")
                                            
                                            # 根据索引获取实际值
                                            if total_idx is not None and total_idx < len(data_array):
                                                quota_total = data_array[total_idx]
                                                quota_remaining = quota_total  # 剩余额度等于总额度
                                            
                                            if used_idx is not None and used_idx < len(data_array):
                                                quota_used = data_array[used_idx]
                                            
                                            print(f"[OK] 额度信息获取成功: 总额度={quota_total}, 已用={quota_used}, 剩余={quota_remaining}", flush=True)
                                            break
                                    
                                    # 查找subscription对象获取时间和订阅信息
                                    for item in data_array:
                                        if isinstance(item, dict) and "plan_name" in item and "status" in item and "current_period_start" in item:
                                            # 获取plan_name的索引并解析实际名称
                                            plan_name_idx = item.get("plan_name")
                                            if plan_name_idx is not None and plan_name_idx < len(data_array):
                                                subscription_type = data_array[plan_name_idx]
                                            else:
                                                subscription_type = "Free"
                                            
                                            # 获取时间戳索引
                                            start_timestamp_idx = item.get("current_period_start")
                                            end_timestamp_idx = item.get("current_period_end")
                                            
                                            # 通过索引获取实际的时间戳值
                                            start_timestamp = None
                                            end_timestamp = None
                                            
                                            if start_timestamp_idx is not None and start_timestamp_idx < len(data_array):
                                                start_timestamp = data_array[start_timestamp_idx]
                                            
                                            if end_timestamp_idx is not None and end_timestamp_idx < len(data_array):
                                                end_timestamp = data_array[end_timestamp_idx]
                                            
                                            # 转换时间戳为datetime对象
                                            if start_timestamp:
                                                register_time = datetime.fromtimestamp(start_timestamp)
                                                print(f"[OK] 注册时间: {register_time.isoformat()}", flush=True)
                                            
                                            if end_timestamp:
                                                expire_time = datetime.fromtimestamp(end_timestamp)
                                                print(f"[OK] 过期时间: {expire_time.isoformat()}", flush=True)
                                                
                                                # 计算试用天数
                                                if start_timestamp and end_timestamp:
                                                    trial_days = int((end_timestamp - start_timestamp) / 86400)
                                            
                                            print(f"[OK] 订阅信息: {subscription_type}, 试用天数: {trial_days}", flush=True)
                                            break
                        else:
                            print(f"[!] 获取额度信息失败，HTTP状态码: {response.status_code}", flush=True)
                    except Exception as e:
                        print(f"[!] 获取额度信息时出错: {e}", flush=True)
                else:
                    print("[!] 未获取到token，跳过额度信息获取", flush=True)

                # 生成唯一 ID
                account_id = str(uuid.uuid4())

                # 如果未从API获取到过期时间，则使用默认计算方式
                if not expire_time:
                    days_to_expire = trial_days if trial_days else 7
                    expire_time = register_time + timedelta(days=days_to_expire)
                    print(f"[*] 使用默认计算: 注册时间 + {days_to_expire} 天", flush=True)
                
                # 如果未计算出试用天数，使用默认值7
                if trial_days is None:
                    trial_days = 7

                account_info = {
                    "id": account_id,
                    "email": email,
                    "password": password,
                    "register_time": register_time.isoformat(),
                    "status": "active",
                    "token": token,
                    "quota_remaining": quota_remaining,
                    "quota_used": quota_used,
                    "quota_total": quota_total,
                    "subscription_type": subscription_type or "Free",
                    "trial_days": trial_days,
                    "expire_time": expire_time.isoformat(),
                    "last_updated": datetime.now().isoformat()
                }

                print("\n" + "="*70, flush=True)
                print("[INFO] 账号信息", flush=True)
                print("="*70, flush=True)
                print(f"ID: {account_id}", flush=True)
                print(f"邮箱: {email}", flush=True)
                print(f"密码: {password}", flush=True)
                print(f"订阅类型: {account_info['subscription_type']}", flush=True)
                print(f"试用天数: {account_info['trial_days']} 天", flush=True)
                print(f"注册时间: {account_info['register_time']}", flush=True)
                print(f"过期时间: {account_info['expire_time']}", flush=True)

                if quota_total is not None and quota_used is not None:
                    print(f"额度信息: 总计 {quota_total} / 已用 {quota_used} / 剩余 {quota_remaining}", flush=True)
                elif quota_total is not None:
                    print(f"额度信息: 总计 {quota_total}", flush=True)
                else:
                    print("额度信息: 未获取", flush=True)

                if token:
                    print(f"Token (前50字符): {token[:50]}...", flush=True)
                    print(f"Token 长度: {len(token)} 字符", flush=True)
                else:
                    print("Token: 未获取", flush=True)
                print("="*70 + "\n", flush=True)

                return account_info
            else:
                print("[X] 注册失败,可能需要人工检查", flush=True)
                return None

        except RuntimeError as e:
            # 浏览器断开连接的专门处理
            error_msg = str(e)
            if "浏览器已被用户关闭" in error_msg or "连接中断" in error_msg:
                print(f"[X] {error_msg}", flush=True)
                print("[X] 注册流程已终止", flush=True)
                return None
            else:
                # 其他运行时错误
                print(f"[X] 注册过程出错: {e}", flush=True)
                traceback.print_exc()
                return None
        except Exception as e:
            print(f"[X] 注册过程出错: {e}", flush=True)
            traceback.print_exc()
            return None

        finally:
            if page:
                try:
                    # 检查浏览器是否还在运行，避免重复关闭
                    if self._check_browser_alive(page):
                        print("[*] 关闭浏览器...", flush=True)
                        page.quit()
                    else:
                        print("[*] 浏览器已关闭，跳过清理步骤", flush=True)
                    
                    # 清理临时用户数据目录
                    if hasattr(page, '_temp_user_data_dir'):
                        temp_dir = page._temp_user_data_dir
                        if os.path.exists(temp_dir):
                            try:
                                shutil.rmtree(temp_dir, ignore_errors=True)
                                print(f"[*] 已清理临时目录: {temp_dir}", flush=True)
                            except:
                                pass  # 忽略清理错误
                                
                except Exception as e:
                    # 忽略关闭浏览器时的错误
                    print(f"[*] 浏览器清理时出现异常（已忽略）: {e}", flush=True)
    
    def batch_register(self, count: int, password: str = "VerdentAI@2024", max_workers: int = 3, use_random_password: bool = False) -> list:
        results = []

        if use_random_password:
            print(f"\n[BATCH] 批量注册模式: 共 {count} 个账号,最大并发数: {max_workers},使用随机密码", flush=True)
        else:
            print(f"\n[BATCH] 批量注册模式: 共 {count} 个账号,最大并发数: {max_workers}", flush=True)

        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            # 如果使用随机密码,每个账户都会生成唯一密码
            futures = [executor.submit(self.register_account, password, use_random_password) for _ in range(count)]

            for i, future in enumerate(as_completed(futures), 1):
                try:
                    result = future.result()
                    if result:
                        results.append(result)
                        print(f"\n[OK] 进度: {i}/{count} - 成功注册: {result['email']}", flush=True)
                    else:
                        print(f"\n[X] 进度: {i}/{count} - 注册失败", flush=True)
                except Exception as e:
                    print(f"\n[X] 进度: {i}/{count} - 异常: {e}", flush=True)

        print(f"\n[SUMMARY] 批量注册完成: 成功 {len(results)}/{count}", flush=True)
        return results


def main():
    if len(sys.argv) < 1:
        print("使用方法:")
        print("  python verdent_auto_register.py [选项]")
        print("\n选项:")
        print("  --count N            批量注册 N 个账号 (默认: 1)")
        print("  --password PASS      设置密码 (默认: VerdentAI@2024)")
        print("  --random-password    为每个账户生成随机密码 (12-16位,包含大小写字母、数字、特殊字符)")
        print("  --workers N          并发数 (默认: 3)")
        print("  --headless           无头模式运行")
        print("  --output FILE        保存结果到 JSON 文件")
        print("\n示例:")
        print("  python verdent_auto_register.py")
        print("  python verdent_auto_register.py --count 5 --workers 2")
        print("  python verdent_auto_register.py --count 3 --random-password")
        print("  python verdent_auto_register.py --count 1 --random-password --output accounts.json")
        sys.exit(1)

    count = 1
    password = "VerdentAI@2024"
    workers = 3
    headless = False
    output_file = None
    use_random_password = False

    i = 1
    while i < len(sys.argv):
        if sys.argv[i] == "--count" and i + 1 < len(sys.argv):
            count = int(sys.argv[i + 1])
            i += 2
        elif sys.argv[i] == "--password" and i + 1 < len(sys.argv):
            password = sys.argv[i + 1]
            i += 2
        elif sys.argv[i] == "--random-password":
            use_random_password = True
            i += 1
        elif sys.argv[i] == "--workers" and i + 1 < len(sys.argv):
            workers = int(sys.argv[i + 1])
            i += 2
        elif sys.argv[i] == "--headless":
            headless = True
            i += 1
        elif sys.argv[i] == "--output" and i + 1 < len(sys.argv):
            output_file = sys.argv[i + 1]
            i += 2
        else:
            i += 1

    # 如果同时指定了 --password 和 --random-password,优先使用随机密码
    if use_random_password:
        print("[*] 已启用随机密码生成功能", flush=True)

    register = VerdentAutoRegister(headless=headless)

    if count == 1:
        result = register.register_account(password, use_random_password)
        results = [result] if result else []
    else:
        results = register.batch_register(count, password, workers, use_random_password)

    if output_file and results:
        with open(output_file, 'w', encoding='utf-8') as f:
            json.dump(results, f, indent=2, ensure_ascii=False)
        print(f"\n[OK] 结果已保存到: {output_file}")

    sys.exit(0 if results else 1)


if __name__ == "__main__":
    main()
