use crate::account_manager::AccountManager;
use crate::models::Account;
use crate::api::VerdentApi;
use crate::jwt_utils;
use crate::pkce::PkceParams;
use crate::settings_manager::{UserSettings, SettingsManager};
use crate::verdent_client::VerdentClientManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader};
use tauri::Manager;

/// 查找自动注册可执行文件
///
/// 返回: (可执行文件路径, 是否为开发模式)
fn find_register_executable(app_handle: &tauri::AppHandle, current_dir: &PathBuf) -> Result<(PathBuf, bool), String> {
    println!("[*] 当前工作目录: {}", current_dir.display());
    println!("[*] 操作系统: {}", std::env::consts::OS);

    // 1. 尝试查找打包的可执行文件 (生产模式)
    // 注意：macOS 不能运行 .exe 文件
    if cfg!(target_os = "windows") {
        // Windows: 查找 .exe 文件
        if let Ok(resource_path) = app_handle.path().resource_dir() {
            println!("[*] Tauri resource_dir: {}", resource_path.display());

            // 尝试多个可能的路径
            let exe_candidates = vec![
                // 直接在 resource_dir 下
                resource_path.join("verdent_auto_register.exe"),
                // 在 resources 子目录下
                resource_path.join("resources").join("verdent_auto_register.exe"),
                // 在 _up_/resources 下 (Tauri 2.0 某些情况)
                resource_path.join("_up_").join("resources").join("verdent_auto_register.exe"),
            ];

            for exe_path in exe_candidates {
                println!("[*] 检查生产模式 exe: {}", exe_path.display());
                if exe_path.exists() {
                    println!("[✓] 找到生产模式 exe 文件: {}", exe_path.display());
                    return Ok((exe_path, false));
                }
            }
        }
    } else if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        // macOS/Linux: 优先查找包装脚本（包含依赖检查）
        // 在打包的应用中，Python 脚本可能在 Resources 目录下
        if let Ok(resource_path) = app_handle.path().resource_dir() {
            println!("[*] Tauri resource_dir: {}", resource_path.display());

            // 优先查找包装脚本（包含自动依赖安装）
            let wrapper_candidates = vec![
                resource_path.join("verdent_auto_register_wrapper.py"),
                resource_path.join("resources").join("verdent_auto_register_wrapper.py"),
                resource_path.join("_up_").join("verdent_auto_register_wrapper.py"),
                resource_path.join("_up_").join("resources").join("verdent_auto_register_wrapper.py"),
            ];

            for wrapper_path in wrapper_candidates {
                println!("[*] 检查包装脚本: {}", wrapper_path.display());
                if wrapper_path.exists() {
                    println!("[✓] 找到包装脚本 (包含依赖检查): {}", wrapper_path.display());
                    return Ok((wrapper_path, true));  // 返回 true 表示是 Python 脚本
                }
            }

            // 如果没有包装脚本，查找主脚本
            let script_candidates = vec![
                resource_path.join("verdent_auto_register.py"),
                resource_path.join("resources").join("verdent_auto_register.py"),
                resource_path.join("_up_").join("verdent_auto_register.py"),
                resource_path.join("_up_").join("resources").join("verdent_auto_register.py"),
            ];

            for script_path in script_candidates {
                println!("[*] 检查主脚本: {}", script_path.display());
                if script_path.exists() {
                    println!("[✓] 找到主脚本: {}", script_path.display());
                    return Ok((script_path, true));  // 返回 true 表示是 Python 脚本
                }
            }
        }
    }

    // 列出 resource_dir 的内容以便调试
    if let Ok(resource_path) = app_handle.path().resource_dir() {
        println!("[*] resource_dir 目录内容:");
        if let Ok(entries) = std::fs::read_dir(&resource_path) {
            for entry in entries.flatten() {
                println!("    - {}", entry.path().display());
            }
        }

        // 同时检查 _up_ 目录的内容
        let up_dir = resource_path.join("_up_");
        if up_dir.exists() {
            println!("[*] _up_ 目录内容:");
            if let Ok(entries) = std::fs::read_dir(&up_dir) {
                for entry in entries.flatten() {
                    println!("    - {}", entry.path().display());
                }
            }
        }
    }

    // 2. 尝试查找 Python 脚本 (开发模式)
    println!("[*] 尝试查找开发模式 Python 脚本...");

    // 先尝试查找包装脚本（包含依赖检查）
    let wrapper_candidates: Vec<Option<PathBuf>> = vec![
        // 当前目录
        Some(current_dir.join("verdent_auto_register_wrapper.py")),
        // 父目录 (src-tauri 的父目录)
        current_dir.parent().map(|p: &std::path::Path| p.join("verdent_auto_register_wrapper.py")),
        // 祖父目录
        current_dir.parent().and_then(|p: &std::path::Path| p.parent()).map(|p: &std::path::Path| p.join("verdent_auto_register_wrapper.py")),
        // macOS: 在 .app/Contents/MacOS 目录（当前目录）已被第一个选项覆盖
        // macOS: 在 .app/Contents 目录
        #[cfg(target_os = "macos")]
        current_dir.parent().map(|p: &std::path::Path| p.join("verdent_auto_register_wrapper.py")),
        // macOS: 在 .app 根目录
        #[cfg(target_os = "macos")]
        current_dir.parent().and_then(|p: &std::path::Path| p.parent()).map(|p: &std::path::Path| p.join("verdent_auto_register_wrapper.py")),
    ];

    for candidate in wrapper_candidates.into_iter().filter_map(|p| p) {
        println!("[*] 检查开发模式包装脚本: {}", candidate.display());
        if candidate.exists() {
            println!("[✓] 找到开发模式包装脚本 (包含依赖检查): {}", candidate.display());
            return Ok((candidate, true));
        }
    }

    // 如果没有包装脚本，查找主脚本
    let script_candidates: Vec<Option<PathBuf>> = vec![
        // 当前目录
        Some(current_dir.join("verdent_auto_register.py")),
        // 父目录 (src-tauri 的父目录)
        current_dir.parent().map(|p: &std::path::Path| p.join("verdent_auto_register.py")),
        // 祖父目录
        current_dir.parent().and_then(|p: &std::path::Path| p.parent()).map(|p: &std::path::Path| p.join("verdent_auto_register.py")),
        // macOS: 在 .app/Contents 目录
        #[cfg(target_os = "macos")]
        current_dir.parent().map(|p: &std::path::Path| p.join("verdent_auto_register.py")),
        // macOS: 在 .app 根目录
        #[cfg(target_os = "macos")]
        current_dir.parent().and_then(|p: &std::path::Path| p.parent()).map(|p: &std::path::Path| p.join("verdent_auto_register.py")),
    ];

    for candidate in script_candidates.into_iter().filter_map(|p| p) {
        println!("[*] 检查开发模式主脚本: {}", candidate.display());
        if candidate.exists() {
            println!("[✓] 找到开发模式主脚本: {}", candidate.display());
            return Ok((candidate, true));
        }
    }

    // 3. 都没找到,返回详细错误信息
    println!("[×] 未找到任何可执行文件!");
    Err(format!(
        "未找到自动注册可执行文件。\n\
        请确保:\n\
        - 开发模式: verdent_auto_register.py 在项目根目录\n\
        - 生产模式: verdent_auto_register.exe 已打包到 resources 目录\n\
        \n\
        调试信息:\n\
        - 当前工作目录: {}\n\
        - 请查看控制台输出的详细路径信息",
        current_dir.display()
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoRegisterRequest {
    pub count: u32,
    pub password: String,
    pub max_workers: u32,
    pub headless: bool,
    pub use_random_password: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AutoRegisterResponse {
    pub success: bool,
    pub registered_count: u32,      // 成功注册的数量
    pub failed_count: u32,           // 失败的数量
    pub total_count: u32,            // 请求的总数量
    pub accounts: Vec<Account>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub token: String,
    pub device_id: Option<String>,
    pub app_version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub access_token: Option<String>,
    pub error: Option<String>,
    pub callback_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginToVSCodeResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetResponse {
    pub success: bool,
    pub deleted_count: usize,
    pub deleted_keys: Vec<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn auto_register_accounts(
    app_handle: tauri::AppHandle,
    request: AutoRegisterRequest
) -> Result<AutoRegisterResponse, String> {
    println!("\n[*] ========== 自动注册账号 ==========");
    println!("[*] 请求参数:");
    println!("    数量: {}", request.count);
    println!("    密码: {}", if request.use_random_password { "随机生成" } else { &request.password });
    println!("    并发数: {}", request.max_workers);
    println!("    无头模式: {}", request.headless);
    println!("    使用随机密码: {}", request.use_random_password);

    // 保存请求的总数量用于统计
    let total_count = request.count;

    // 获取当前工作目录
    let current_dir = std::env::current_dir().map_err(|e| e.to_string())?;
    println!("[*] 当前工作目录: {}", current_dir.display());

    // 检测运行环境并查找可执行文件
    let (executable_path, is_dev_mode) = find_register_executable(&app_handle, &current_dir)?;

    println!("[✓] 运行模式: {}", if is_dev_mode { "开发模式" } else { "生产模式" });
    println!("[✓] 可执行文件: {}", executable_path.display());

    // 生成输出文件路径
    let output_file = std::env::temp_dir().join(format!("verdent_register_{}.json", uuid::Uuid::new_v4()));
    println!("[*] 输出文件: {}", output_file.display());

    // 构建命令 - 根据运行模式选择不同的执行方式
    let mut cmd = if is_dev_mode {
        // 开发模式: 使用 python 解释器运行 .py 脚本
        // macOS 默认使用 python3，Windows 使用 python
        let python_cmd = if cfg!(target_os = "macos") {
            "python3"
        } else {
            "python"
        };
        let mut c = Command::new(python_cmd);
        c.arg(&executable_path);
        c
    } else {
        // 生产模式: 直接运行打包的 .exe 文件
        Command::new(&executable_path)
    };

    // 添加通用参数
    cmd.arg("--count")
        .arg(request.count.to_string())
        .arg("--workers")
        .arg(request.max_workers.to_string())
        .arg("--output")
        .arg(&output_file)
        .env("PYTHONIOENCODING", "utf-8");  // 设置输出编码为 UTF-8

    // 如果使用随机密码,添加 --random-password 参数
    if request.use_random_password {
        cmd.arg("--random-password");
        println!("[*] 已启用随机密码生成");
    } else {
        // 只有在不使用随机密码时才传递固定密码
        cmd.arg("--password").arg(&request.password);
    }

    if request.headless {
        cmd.arg("--headless");
    }

    // 打印完整命令
    let password_arg = if request.use_random_password {
        " --random-password".to_string()
    } else {
        format!(" --password {}", request.password)
    };

    let headless_arg = if request.headless { " --headless" } else { "" };

    if is_dev_mode {
        println!("[*] 执行命令: python {} --count {} --workers {} --output {}{}{}",
            executable_path.display(),
            request.count,
            request.max_workers,
            output_file.display(),
            password_arg,
            headless_arg
        );
    } else {
        println!("[*] 执行命令: {} --count {} --workers {} --output {}{}{}",
            executable_path.display(),
            request.count,
            request.max_workers,
            output_file.display(),
            password_arg,
            headless_arg
        );
    }

    // 执行命令 - 使用 spawn 实现实时输出
    println!("[*] 开始执行 Python 脚本...");

    // 配置子进程的 stdio
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped());

    // Windows: 隐藏子进程控制台窗口
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = cmd.spawn().map_err(|e| {
        eprintln!("[×] 启动 Python 进程失败: {}", e);
        e.to_string()
    })?;

    // 获取 stdout 和 stderr 的句柄
    let stdout = child.stdout.take().ok_or("无法获取 stdout")?;
    let stderr = child.stderr.take().ok_or("无法获取 stderr")?;

    // 创建 BufReader 用于逐行读取
    let stdout_reader = BufReader::new(stdout);
    let stderr_reader = BufReader::new(stderr);

    // 实时打印 stdout
    println!("[*] Python 脚本实时输出:");
    println!("----------------------------------------");

    // 使用线程同时读取 stdout 和 stderr
    let stdout_handle = std::thread::spawn(move || {
        let mut stdout_output = String::new();
        for line in stdout_reader.lines() {
            if let Ok(line) = line {
                println!("{}", line);
                // 同时收集输出用于错误检测
                stdout_output.push_str(&line);
                stdout_output.push('\n');
            }
        }
        stdout_output
    });

    let stderr_handle = std::thread::spawn(move || {
        let mut stderr_output = String::new();
        for line in stderr_reader.lines() {
            if let Ok(line) = line {
                eprintln!("[STDERR] {}", line);
                stderr_output.push_str(&line);
                stderr_output.push('\n');
            }
        }
        stderr_output
    });

    // 等待输出线程完成
    let stdout_output = stdout_handle.join().unwrap_or_default();
    let stderr_output = stderr_handle.join().unwrap_or_default();

    // 等待进程结束
    let status = child.wait().map_err(|e| {
        eprintln!("[×] 等待进程结束失败: {}", e);
        e.to_string()
    })?;

    println!("----------------------------------------");

    // 检查执行状态
    if !status.success() {
        eprintln!("[×] Python 脚本执行失败!");
        eprintln!("    退出码: {:?}", status.code());
        if !stderr_output.is_empty() {
            eprintln!("    错误输出:");
            eprintln!("{}", stderr_output);
        }

        // 检测浏览器断开连接的特定错误（同时检查 stdout 和 stderr）
        let combined_output = format!("{}\n{}", stdout_output, stderr_output);
        let error_msg = if combined_output.contains("浏览器已被用户关闭") ||
                           combined_output.contains("连接中断") ||
                           combined_output.contains("浏览器连接已断开") {
            eprintln!("[×] 检测到浏览器断开连接");
            "用户手动关闭了浏览器，注册流程已终止".to_string()
        } else {
            format!("注册脚本执行失败 (退出码: {:?})\n{}", status.code(), stderr_output)
        };

        return Ok(AutoRegisterResponse {
            success: false,
            registered_count: 0,
            failed_count: total_count,
            total_count,
            accounts: Vec::new(),
            error: Some(error_msg),
        });
    }

    println!("[✓] Python 脚本执行成功");

    // 读取输出文件
    let accounts: Vec<Account> = if output_file.exists() {
        println!("[*] 读取输出文件: {}", output_file.display());
        let content = std::fs::read_to_string(&output_file).map_err(|e| {
            eprintln!("[×] 读取输出文件失败: {}", e);
            e.to_string()
        })?;
        println!("[*] 文件内容长度: {} 字节", content.len());

        serde_json::from_str(&content).map_err(|e| {
            eprintln!("[×] 解析 JSON 失败: {}", e);
            eprintln!("    文件内容: {}", content);
            e.to_string()
        })?
    } else {
        eprintln!("[×] 警告: 输出文件不存在");
        Vec::new()
    };

    println!("[*] 解析到 {} 个账号", accounts.len());

    // 保存账号到数据库
    let account_mgr = AccountManager::new().map_err(|e| {
        eprintln!("[×] 创建账号管理器失败: {}", e);
        e.to_string()
    })?;
    let mut saved_accounts = Vec::new();

    // 创建 API 客户端用于刷新账户信息
    let api = VerdentApi::new();

    for (i, account) in accounts.iter().enumerate() {
        println!("[*] 保存账号 {}/{}: {}", i + 1, accounts.len(), account.email);
        match account_mgr.add_account(account.clone()) {
            Ok(mut saved) => {
                println!("    ✓ 保存成功");

                // 检查账户信息是否完整
                let has_complete_info = saved.quota_total.is_some()
                    && saved.quota_used.is_some()
                    && saved.quota_remaining.is_some()
                    && saved.subscription_type.is_some();

                if has_complete_info {
                    // 信息已完整,仅补充 Token 过期时间和更新时间
                    println!("    [*] 账户信息已完整,无需刷新");

                    if let Some(token) = &saved.token {
                        saved.token_expire_time = jwt_utils::extract_token_expire_time(token);
                    }

                    saved.last_updated = Some(chrono::Local::now().to_rfc3339());

                    // 保存更新后的账户
                    match account_mgr.update_account(&saved.id, saved.clone()) {
                        Ok(updated) => {
                            println!("    [✓] 账户信息已更新并保存");
                            saved = updated;
                        }
                        Err(e) => {
                            eprintln!("    [!] 保存更新后的账户失败: {}", e);
                        }
                    }
                } else if let Some(token) = &saved.token {
                    // 信息不完整,需要刷新
                    println!("    [*] 账户信息不完整,立即刷新...");
                    match api.fetch_user_info_with_retry(token, 3).await {
                        Ok(user_info) => {
                            println!("    [✓] 成功获取用户信息");

                            // 计算额度信息
                            let quota_consumed = user_info.token_info.as_ref()
                                .and_then(|ti| ti.token_consumed)
                                .unwrap_or(0.0);
                            let quota_free = user_info.token_info.as_ref()
                                .and_then(|ti| ti.token_free)
                                .unwrap_or(0.0);
                            let quota_remaining = quota_free - quota_consumed;

                            // 更新额度信息
                            saved.quota_remaining = Some(format!("{:.2}", quota_remaining));
                            saved.quota_used = Some(format!("{:.2}", quota_consumed));
                            saved.quota_total = Some(format!("{:.2}", quota_free));

                            // 更新订阅信息 - 优先使用 planName,其次 levelName,最后使用顶层 subscription_type
                            saved.subscription_type = user_info.subscription_info.as_ref()
                                .and_then(|si| si.plan_name.clone())
                                .or_else(|| user_info.subscription_info.as_ref()
                                    .and_then(|si| si.level_name.clone()))
                                .or(user_info.subscription_type);

                            // trial_days 从 UserInfo 顶层获取
                            saved.trial_days = user_info.trial_days;

                            // 从 subscriptionInfo 中提取订阅周期结束时间和自动续订状态
                            let current_period_end = user_info.subscription_info.as_ref()
                                .and_then(|si| si.current_period_end);
                            saved.current_period_end = current_period_end;

                            saved.auto_renew = user_info.subscription_info.as_ref()
                                .and_then(|si| si.auto_renew);

                            // 计算订阅到期时间
                            saved.expire_time = if let Some(period_end) = current_period_end {
                                let dt = chrono::DateTime::from_timestamp(period_end, 0)
                                    .unwrap_or_else(|| chrono::Utc::now());
                                Some(dt.to_rfc3339())
                            } else {
                                None
                            };

                            // 从 JWT Token 中提取 Token 过期时间
                            saved.token_expire_time = jwt_utils::extract_token_expire_time(token);

                            saved.last_updated = Some(chrono::Local::now().to_rfc3339());

                            // 保存更新后的账户
                            match account_mgr.update_account(&saved.id, saved.clone()) {
                                Ok(updated) => {
                                    println!("    [✓] 账户信息已刷新并保存");
                                    saved = updated;
                                }
                                Err(e) => {
                                    eprintln!("    [!] 保存刷新后的账户失败: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("    [!] 刷新账户信息失败: {}", e);
                            eprintln!("    [*] 账户已保存,但信息未刷新,可稍后手动刷新");
                        }
                    }
                } else {
                    println!("    [!] 账户没有 Token,跳过刷新");
                }

                saved_accounts.push(saved);
            },
            Err(e) => {
                eprintln!("    × 保存失败: {}", e);
            }
        }
    }

    // 清理临时文件
    if output_file.exists() {
        println!("[*] 清理临时文件: {}", output_file.display());
        let _ = std::fs::remove_file(&output_file);
    }

    let success_count = saved_accounts.len() as u32;
    let failed_count = accounts.len() as u32 - success_count;

    println!("[✓] 自动注册完成: 成功 {}/{} 个账号, 失败 {} 个", success_count, accounts.len(), failed_count);
    println!("[*] ====================================\n");

    Ok(AutoRegisterResponse {
        success: success_count > 0,  // 至少成功一个就算成功
        registered_count: success_count,
        failed_count,
        total_count,
        accounts: saved_accounts,
        error: None,
    })
}

#[tauri::command]
pub async fn get_all_accounts() -> Result<Vec<Account>, String> {
    let mgr = AccountManager::new().map_err(|e| e.to_string())?;
    mgr.get_all_accounts().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_data_path() -> Result<String, String> {
    let mgr = AccountManager::new().map_err(|e| e.to_string())?;
    Ok(mgr.get_storage_path())
}

#[tauri::command]
pub async fn get_account(id: String) -> Result<Account, String> {
    let account_mgr = AccountManager::new().map_err(|e| e.to_string())?;
    account_mgr.get_account(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(id: String, account: Account) -> Result<Account, String> {
    let account_mgr = AccountManager::new().map_err(|e| e.to_string())?;
    account_mgr.update_account(&id, account).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_account(id: String) -> Result<bool, String> {
    let account_mgr = AccountManager::new().map_err(|e| e.to_string())?;
    account_mgr.delete_account(&id).map_err(|e| e.to_string())?;
    Ok(true)
}

#[tauri::command]
pub async fn update_account_token(id: String, token: String) -> Result<Account, String> {
    let account_mgr = AccountManager::new().map_err(|e| e.to_string())?;
    account_mgr.update_account_token(&id, token).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account_quota(id: String, remaining: i32, total: i32) -> Result<Account, String> {
    let account_mgr = AccountManager::new().map_err(|e| e.to_string())?;
    account_mgr.update_account_quota(&id, remaining, total).map_err(|e| e.to_string())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageInfo {
    pub path: String,
    pub keys: Vec<String>,
}

#[tauri::command]
pub async fn login_with_token(request: LoginRequest) -> Result<LoginResponse, String> {
    // 步骤 1: 生成 PKCE 参数
    let pkce = PkceParams::generate();

    // 调试日志: 输出 PKCE 参数
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    State 长度: {} 字符", pkce.state.len());
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    // 步骤 2: 使用 token 请求授权码
    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&request.token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginResponse {
                success: false,
                access_token: None,
                error: Some(format!("获取授权码失败: {}", e)),
                callback_url: None,
            });
        }
    };

    // 步骤 3: 使用授权码交换访问令牌
    println!("\n[*] 步骤 3: 交换访问令牌");
    let token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功!");
            println!("    访问令牌 (前20字符): {}...", &token.chars().take(20).collect::<String>());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginResponse {
                success: false,
                access_token: None,
                error: Some(format!("交换访问令牌失败: {}", e)),
                callback_url: None,
            });
        }
    };

    // 步骤 4: 构建 VS Code 回调 URL
    let callback_url = format!("vscode://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);

    // 调试日志: 输出完整的回调 URL
    println!("[*] 构建 VS Code 回调 URL:");
    println!("    {}", callback_url);
    println!("    URL 长度: {} 字符", callback_url.len());

    // 步骤 5: 保存到本地存储
    use crate::storage::StorageManager;
    let storage = StorageManager::new().map_err(|e| e.to_string())?;

    // 保存访问令牌
    storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": token
    })).map_err(|e| e.to_string())?;

    // 保存 API 提供商
    storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })).map_err(|e| e.to_string())?;

    // 如果提供了设备 ID,保存它
    if let Some(device_id) = request.device_id {
        storage.save("device_id", serde_json::json!({
            "value": device_id
        })).map_err(|e| e.to_string())?;
    }

    // 如果提供了应用版本,保存它
    if let Some(app_version) = request.app_version {
        storage.save("app_version", serde_json::json!({
            "value": app_version
        })).map_err(|e| e.to_string())?;
    }

    Ok(LoginResponse {
        success: true,
        access_token: Some(token),
        error: None,
        callback_url: Some(callback_url),
    })
}

#[tauri::command]
pub async fn reset_device_identity(generate_new_device_id: bool, selected_editors: Option<Vec<String>>) -> Result<ResetResponse, String> {
    use crate::storage::StorageManager;
    use crate::vscode_storage::VSCodeStorageManager;
    use crate::editor_config::EditorType;

    println!("[*] 开始重置设备身份...");
    println!("[*] 生成新设备ID: {}", generate_new_device_id);

    let mut deleted_keys = Vec::new();
    let mut deleted_count = 0;

    // 1. 清除应用存储（Verdent相关的认证信息）
    println!("[*] 步骤 1: 清理 Verdent 认证信息...");
    let storage = StorageManager::new().map_err(|e| {
        eprintln!("[×] 创建 StorageManager 失败: {}", e);
        e.to_string()
    })?;

    // 获取当前存储的所有键
    let keys_before = storage.list_keys().map_err(|e| {
        eprintln!("[×] 获取存储键列表失败: {}", e);
        e.to_string()
    })?;

    deleted_count += keys_before.len();
    deleted_keys.extend(keys_before.clone());

    // 清除所有存储
    storage.clear().map_err(|e| {
        eprintln!("[×] 清除存储失败: {}", e);
        e.to_string()
    })?;

    println!("[✓] Verdent 认证信息已清理: {} 项", keys_before.len());

    // 2. 重置编辑器设备标识
    if generate_new_device_id {
        println!("[*] 步骤 2: 重置编辑器设备标识...");
        
        // 解析选中的编辑器列表
        let editors_to_reset = if let Some(ref editor_names) = selected_editors {
            let mut editors = Vec::new();
            for name in editor_names {
                if let Some(editor) = EditorType::from_string(&name) {
                    editors.push(editor);
                }
            }
            editors
        } else {
            // 默认只重置 VS Code
            vec![EditorType::VSCode]
        };
        
        // 重置每个编辑器的设备标识
        for editor in &editors_to_reset {
            println!("[*] 重置 {} 设备标识...", editor.display_name());
            
            // 使用 VSCodeStorageManager 处理所有编辑器
            match VSCodeStorageManager::new_with_editor(editor.folder_name()) {
                Ok(storage_manager) => {
                    match storage_manager.reset_telemetry_ids() {
                        Ok((sqm_id, device_id, machine_id)) => {
                            deleted_keys.push(format!("{} telemetry.sqmId (重置为: {})", editor.display_name(), sqm_id));
                            deleted_keys.push(format!("{} telemetry.devDeviceId (重置为: {})", editor.display_name(), device_id));
                            deleted_keys.push(format!("{} telemetry.machineId (重置为: {})", editor.display_name(), machine_id));
                            deleted_count += 3;
                            println!("[✓] {} 设备标识已重置", editor.display_name());
                            println!("    新 sqmId: {}", sqm_id);
                            println!("    新 deviceId: {}", device_id);
                            println!("    新 machineId: {}", machine_id);
                        }
                        Err(e) => {
                            eprintln!("[!] 重置 {} 设备标识失败: {}", editor.display_name(), e);
                            println!("[!] 跳过 {} 设备标识重置", editor.display_name());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[!] 创建 {} StorageManager 失败: {}", editor.display_name(), e);
                    println!("[!] 跳过 {} 设备标识重置", editor.display_name());
                }
            }
        }
    } else {
        println!("[*] 跳过编辑器设备标识重置（根据参数设置）");
    }

    // 3. 重置编辑器 state.vscdb 中的 deviceId
    if generate_new_device_id {
        println!("[*] 步骤 3: 重置编辑器 state.vscdb 中的 deviceId...");
        use crate::cleanup_utils::reset_editors_device_id;
        
        // 使用与步骤2相同的编辑器列表
        let editors_for_db = if let Some(ref editor_names) = selected_editors {
            let mut editors = Vec::new();
            for name in editor_names {
                if let Some(editor) = EditorType::from_string(&name) {
                    editors.push(editor);
                }
            }
            editors
        } else {
            vec![EditorType::VSCode]
        };
        
        let results = reset_editors_device_id(&editors_for_db, true);
        for (editor_name, result) in results {
            match result {
                Ok(new_device_id) => {
                    deleted_keys.push(format!("{} state.vscdb deviceId (重置为: {})", editor_name, new_device_id));
                    deleted_count += 1;
                    println!("[✓] {} state.vscdb deviceId 已重置: {}", editor_name, new_device_id);
                }
                Err(e) => {
                    eprintln!("[!] 重置 {} state.vscdb deviceId 失败: {}", editor_name, e);
                    println!("[!] 跳过 {} state.vscdb deviceId 重置", editor_name);
                }
            }
        }
    } else {
        println!("[*] 跳过编辑器 state.vscdb deviceId 重置（根据参数设置）");
    }

    // 4. 重置机器码（仅 Windows）
    if generate_new_device_id && cfg!(target_os = "windows") {
        println!("[*] 步骤 4: 重置机器码...");
        use crate::machine_guid::MachineGuidManager;
        
        match MachineGuidManager::new() {
            Ok(manager) => {
                match manager.reset_to_new_guid() {
                    Ok(new_guid) => {
                        deleted_keys.push(format!("Windows MachineGuid (重置为: {})", new_guid));
                        deleted_count += 1;
                        println!("[✓] 机器码已重置: {}", new_guid);
                    }
                    Err(e) => {
                        eprintln!("[!] 重置机器码失败: {}", e);
                        println!("[!] 提示: 可能需要管理员权限");
                    }
                }
            }
            Err(e) => {
                eprintln!("[!] 创建 MachineGuidManager 失败: {}", e);
                println!("[!] 跳过机器码重置");
            }
        }
    } else if generate_new_device_id && !cfg!(target_os = "windows") {
        println!("[*] 跳过机器码重置（非 Windows 系统）");
    }

    println!("[✓] 设备身份重置完成");
    println!("[✓] 处理了 {} 个存储项", deleted_count);

    Ok(ResetResponse {
        success: true,
        deleted_count,
        deleted_keys,
        error: None,
    })
}

#[tauri::command]
pub async fn reset_all_storage(generate_new_device_id: bool, selected_editors: Option<Vec<String>>) -> Result<ResetResponse, String> {
    use crate::storage::StorageManager;
    use crate::vscode_storage::VSCodeStorageManager;
    use crate::editor_config::EditorType;

    println!("[*] 开始完全清理所有存储...");
    println!("[*] 生成新设备ID: {}", generate_new_device_id);

    let mut deleted_keys = Vec::new();
    let mut deleted_count = 0;

    // 1. 清除应用存储
    println!("[*] 步骤 1: 清除应用存储...");
    let storage = StorageManager::new().map_err(|e| {
        eprintln!("[×] 创建 StorageManager 失败: {}", e);
        e.to_string()
    })?;

    let keys_before = storage.list_keys().map_err(|e| {
        eprintln!("[×] 获取存储键列表失败: {}", e);
        e.to_string()
    })?;

    deleted_count += keys_before.len();
    deleted_keys.extend(keys_before.clone());

    storage.clear().map_err(|e| {
        eprintln!("[×] 清除应用存储失败: {}", e);
        e.to_string()
    })?;

    println!("[✓] 应用存储已清除: {} 项", keys_before.len());

    // 2. 重置编辑器设备标识
    if generate_new_device_id {
        println!("[*] 步骤 2: 重置编辑器设备标识...");
        
        // 解析选中的编辑器列表
        let editors_to_reset = if let Some(editor_names) = selected_editors.clone() {
            let mut editors = Vec::new();
            for name in editor_names {
                if let Some(editor) = EditorType::from_string(&name) {
                    editors.push(editor);
                }
            }
            editors
        } else {
            // 默认只重置 VS Code
            vec![EditorType::VSCode]
        };
        
        // 重置每个编辑器的设备标识
        for editor in &editors_to_reset {
            println!("[*] 重置 {} 设备标识...", editor.display_name());
            
            // 使用 VSCodeStorageManager 处理所有编辑器
            match VSCodeStorageManager::new_with_editor(editor.folder_name()) {
                Ok(storage_manager) => {
                    match storage_manager.reset_telemetry_ids() {
                        Ok((sqm_id, device_id, machine_id)) => {
                            deleted_keys.push(format!("{} telemetry.sqmId (重置为: {})", editor.display_name(), sqm_id));
                            deleted_keys.push(format!("{} telemetry.devDeviceId (重置为: {})", editor.display_name(), device_id));
                            deleted_keys.push(format!("{} telemetry.machineId (重置为: {})", editor.display_name(), machine_id));
                            deleted_count += 3;
                            println!("[✓] {} 设备标识已重置", editor.display_name());
                            println!("    新 sqmId: {}", sqm_id);
                            println!("    新 deviceId: {}", device_id);
                            println!("    新 machineId: {}", machine_id);
                        }
                        Err(e) => {
                            eprintln!("[!] 重置 {} 设备标识失败: {}", editor.display_name(), e);
                            println!("[!] 跳过 {} 设备标识重置", editor.display_name());
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[!] 创建 {} StorageManager 失败: {}", editor.display_name(), e);
                    println!("[!] 跳过 {} 设备标识重置", editor.display_name());
                }
            }
        }
    } else {
        println!("[*] 跳过编辑器设备标识重置（根据参数设置）");
    }

    // 3. 清空 Verdent 相关文件夹内容（不删除账户数据）
    println!("[*] 步骤 3: 清空 Verdent 相关文件夹...");
    {
        // 解析选中的编辑器列表（如果没有提供，默认清理所有编辑器）
        let editors_to_clean = if let Some(ref editor_names) = selected_editors {
            let mut editors = Vec::new();
            for name in editor_names {
                if let Some(editor) = EditorType::from_string(&name) {
                    editors.push(editor);
                }
            }
            editors
        } else {
            // 默认只清理 VS Code
            vec![EditorType::VSCode]
        };
        
        use crate::cleanup_utils::clear_verdent_directories_for_editors;
        let (dir_count, cleaned_paths) = clear_verdent_directories_for_editors(&editors_to_clean);
        deleted_count += dir_count;
        deleted_keys.extend(cleaned_paths);
        println!("[✓] 已清理 {} 个文件/文件夹", dir_count);
    }

    // 3.5 删除用户目录下的 .verdent 文件夹
    println!("[*] 步骤 3.5: 删除用户目录下的 .verdent 文件夹...");
    {
        use crate::cleanup_utils::delete_verdent_home_directory;
        let (dir_count, deleted_paths) = delete_verdent_home_directory();
        deleted_count += dir_count;
        deleted_keys.extend(deleted_paths);
    }

    // 4. 重置编辑器 state.vscdb 中的 deviceId
    if generate_new_device_id {
        println!("[*] 步骤 4: 重置编辑器 state.vscdb 中的 deviceId...");
        use crate::cleanup_utils::reset_editors_device_id;
        
        // 使用与步骤2相同的编辑器列表
        let editors_for_db = if let Some(ref editor_names) = selected_editors {
            let mut editors = Vec::new();
            for name in editor_names {
                if let Some(editor) = EditorType::from_string(&name) {
                    editors.push(editor);
                }
            }
            editors
        } else {
            vec![EditorType::VSCode]
        };
        
        let results = reset_editors_device_id(&editors_for_db, true);
        for (editor_name, result) in results {
            match result {
                Ok(new_device_id) => {
                    deleted_keys.push(format!("{} state.vscdb deviceId (重置为: {})", editor_name, new_device_id));
                    deleted_count += 1;
                    println!("[✓] {} state.vscdb deviceId 已重置: {}", editor_name, new_device_id);
                }
                Err(e) => {
                    eprintln!("[!] 重置 {} state.vscdb deviceId 失败: {}", editor_name, e);
                    println!("[!] 跳过 {} state.vscdb deviceId 重置", editor_name);
                }
            }
        }
    } else {
        println!("[*] 跳过编辑器 state.vscdb deviceId 重置（根据参数设置）");
    }

    // 5. 重置机器码（仅 Windows）
    if generate_new_device_id && cfg!(target_os = "windows") {
        println!("[*] 步骤 5: 重置机器码（MachineGuid）...");
        use crate::machine_guid::MachineGuidManager;
        
        match MachineGuidManager::new() {
            Ok(manager) => {
                match manager.reset_to_new_guid() {
                    Ok(new_guid) => {
                        deleted_keys.push(format!("Windows MachineGuid (重置为: {})", new_guid));
                        deleted_count += 1;
                        println!("[✓] 机器码已重置: {}", new_guid);
                    }
                    Err(e) => {
                        eprintln!("[!] 重置机器码失败: {}", e);
                        println!("[!] 提示: 可能需要管理员权限");
                    }
                }
            }
            Err(e) => {
                eprintln!("[!] 创建 MachineGuidManager 失败: {}", e);
                println!("[!] 跳过机器码重置");
            }
        }
    } else if generate_new_device_id && !cfg!(target_os = "windows") {
        println!("[*] 跳过机器码重置（非 Windows 系统）");
    }

    println!("[✓] 完全清理完成");
    println!("[✓] 总共清除了 {} 个存储项", deleted_count);

    Ok(ResetResponse {
        success: true,
        deleted_count,
        deleted_keys,
        error: None,
    })
}

#[tauri::command]
pub async fn get_storage_info() -> Result<StorageInfo, String> {
    use crate::storage::StorageManager;
    let storage = StorageManager::new().map_err(|e| e.to_string())?;

    Ok(StorageInfo {
        path: storage.get_path().to_string_lossy().to_string(),
        keys: storage.list_keys().map_err(|e| e.to_string())?,
    })
}

#[tauri::command]
pub async fn open_vscode_callback(callback_url: String) -> Result<bool, String> {
    // 调试日志: 输出接收到的回调 URL
    println!("[*] 打开 VS Code 回调链接:");
    println!("    接收到的 URL: {}", callback_url);
    println!("    URL 长度: {} 字符", callback_url.len());

    // 检查 URL 中是否包含 state 参数
    if callback_url.contains("&state=") {
        println!("    ✓ URL 包含 state 参数");
    } else {
        println!("    ✗ 警告: URL 缺少 state 参数!");
    }

    // 使用系统默认方式打开 URL (会触发 VS Code 的 URL handler)
    #[cfg(target_os = "windows")]
    {
        println!("    使用 Windows 命令打开...");

        // 方法1: 使用 PowerShell Start-Process (推荐,更可靠)
        let result = Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-Command",
                &format!("Start-Process '{}'", callback_url)
            ])
            .spawn();

        match result {
            Ok(_) => {
                println!("    ✓ 使用 PowerShell 成功打开");
            },
            Err(e) => {
                println!("    ✗ PowerShell 失败: {}, 尝试备用方法...", e);
                // 备用方法: 使用 explorer
                Command::new("explorer")
                    .arg(&callback_url)
                    .spawn()
                    .map_err(|e| format!("打开回调链接失败: {}", e))?;
                println!("    ✓ 使用 explorer 成功打开");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        Command::new("open")
            .arg(&callback_url)
            .spawn()
            .map_err(|e| format!("打开回调链接失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        Command::new("xdg-open")
            .arg(&callback_url)
            .spawn()
            .map_err(|e| format!("打开回调链接失败: {}", e))?;
    }

    println!("[✓] VS Code 回调链接已发送到系统");
    Ok(true)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VSCodeIdsInfo {
    pub success: bool,
    pub storage_path: String,
    pub sqm_id: Option<String>,
    pub device_id: Option<String>,
    pub machine_id: Option<String>,
    pub extension_version: Option<String>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn get_vscode_ids() -> Result<VSCodeIdsInfo, String> {
    use crate::vscode_storage::VSCodeStorageManager;

    println!("[*] 读取 VS Code 设备信息...");

    let vscode_storage = match VSCodeStorageManager::new() {
        Ok(storage) => storage,
        Err(e) => {
            eprintln!("[×] 创建 VSCodeStorageManager 失败: {}", e);
            return Ok(VSCodeIdsInfo {
                success: false,
                storage_path: "未知".to_string(),
                sqm_id: None,
                device_id: None,
                machine_id: None,
                extension_version: None,
                error: Some(format!("初始化失败: {}", e)),
            });
        }
    };

    let storage_path = vscode_storage.get_storage_path();
    println!("[*] VS Code 存储路径: {}", storage_path);

    // 获取扩展版本
    let extension_version = match vscode_storage.get_verdent_extension_version() {
        Ok(version) => {
            println!("[✓] Verdent 扩展版本: {}", version);
            Some(version)
        },
        Err(e) => {
            println!("[!] 未找到 Verdent 扩展: {}", e);
            None
        }
    };

    // 获取设备 IDs
    match vscode_storage.get_current_ids() {
        Ok((sqm_id, device_id, machine_id)) => {
            println!("[✓] 成功读取设备信息:");
            if let Some(ref id) = sqm_id {
                println!("    sqmId: {}", id);
            } else {
                println!("    sqmId: 未设置");
            }
            if let Some(ref id) = device_id {
                println!("    devDeviceId: {}", id);
            } else {
                println!("    devDeviceId: 未设置");
            }
            if let Some(ref id) = machine_id {
                println!("    machineId: {}", id);
            } else {
                println!("    machineId: 未设置");
            }

            Ok(VSCodeIdsInfo {
                success: true,
                storage_path,
                sqm_id,
                device_id,
                machine_id,
                extension_version,
                error: None,
            })
        },
        Err(e) => {
            eprintln!("[×] 读取设备信息失败: {}", e);
            Ok(VSCodeIdsInfo {
                success: false,
                storage_path,
                sqm_id: None,
                device_id: None,
                machine_id: None,
                extension_version,
                error: Some(format!("读取失败: {}", e)),
            })
        }
    }
}

#[tauri::command]
pub async fn reset_vscode_ids() -> Result<bool, String> {
    use crate::vscode_storage::VSCodeStorageManager;

    println!("[*] 重置 VS Code 设备标识...");

    let vscode_storage = VSCodeStorageManager::new().map_err(|e| {
        eprintln!("[×] 创建 VSCodeStorageManager 失败: {}", e);
        e.to_string()
    })?;

    println!("[*] VS Code 存储路径: {}", vscode_storage.get_storage_path());

    vscode_storage.clear().map_err(|e| {
        eprintln!("[×] 清除设备标识失败: {}", e);
        e.to_string()
    })?;

    println!("[✓] VS Code 设备标识已清除");
    Ok(true)
}

/// 一键登录到 VS Code
/// 使用账户的 token 直接登录,无需手动输入
#[tauri::command]
pub async fn login_to_vscode(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 VS Code ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    // 步骤 1: 生成 PKCE 参数
    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    // 步骤 2: 使用 token 请求授权码
    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    // 步骤 3: 交换访问令牌
    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    // 步骤 4: 构建 VS Code 回调 URL
    let callback_url = format!("vscode://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 VS Code 回调 URL:");
    println!("    {}", callback_url);

    // 步骤 5: 保存到本地存储
    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    // 保存访问令牌
    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    // 保存 API 提供商
    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");

    // 步骤 6: 打开 VS Code 回调 URL
    println!("[*] 正在打开 VS Code...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        // 在 Windows 下,使用 rundll32 是最可靠的方式
        // 它直接调用 Windows URL 处理器,不受特殊字符影响
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 VS Code 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 VS Code 失败。请确保 VS Code 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 VS Code 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 VS Code 失败。请确保 VS Code 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 VS Code 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 VS Code 失败。请确保 VS Code 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] VS Code 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

/// 一键登录到 Windsurf
/// 使用账户的 token 直接登录,无需手动输入
#[tauri::command]
pub async fn login_to_windsurf(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 Windsurf ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    // 步骤 1: 生成 PKCE 参数
    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    // 步骤 2: 使用 token 请求授权码
    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    // 步骤 3: 交换访问令牌
    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    // 步骤 4: 构建 Windsurf 回调 URL
    // 使用 windsurf:// 协议替代 vscode://
    let callback_url = format!("windsurf://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 Windsurf 回调 URL:");
    println!("    {}", callback_url);

    // 步骤 5: 保存到本地存储 (Windsurf 的存储路径可能不同)
    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    // 保存访问令牌
    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    // 保存 API 提供商
    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");

    // 步骤 6: 打开 Windsurf 回调 URL
    println!("[*] 正在打开 Windsurf...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        // 在 Windows 下,使用 rundll32 是最可靠的方式
        // 它直接调用 Windows URL 处理器,不受特殊字符影响
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 Windsurf 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Windsurf 失败。请确保 Windsurf 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Windsurf 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Windsurf 失败。请确保 Windsurf 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Windsurf 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Windsurf 失败。请确保 Windsurf 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] Windsurf 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

/// 一键登录到 Cursor
/// 使用账户的 token 直接登录,无需手动输入
#[tauri::command]
pub async fn login_to_cursor(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 Cursor ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    // 步骤 1: 生成 PKCE 参数
    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    // 步骤 2: 使用 token 请求授权码
    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    // 步骤 3: 交换访问令牌
    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    // 步骤 4: 构建 Cursor 回调 URL
    // 使用 cursor:// 协议
    let callback_url = format!("cursor://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 Cursor 回调 URL:");
    println!("    {}", callback_url);

    // 步骤 5: 保存到本地存储 (Cursor 的存储路径可能不同)
    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    // 保存访问令牌
    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    // 保存 API 提供商
    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");

    // 步骤 6: 打开 Cursor 回调 URL
    println!("[*] 正在打开 Cursor...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        // 在 Windows 下,使用 rundll32 是最可靠的方式
        // 它直接调用 Windows URL 处理器,不受特殊字符影响
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 Cursor 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Cursor 失败。请确保 Cursor 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Cursor 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Cursor 失败。请确保 Cursor 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Cursor 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Cursor 失败。请确保 Cursor 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] Cursor 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

#[tauri::command]
pub async fn login_to_trae(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 Trae ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    // 步骤 1: 生成 PKCE 参数
    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    // 步骤 2: 使用 token 请求授权码
    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    // 步骤 3: 交换访问令牌
    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    // 步骤 4: 构建 Trae 回调 URL
    // 使用 trae:// 协议
    let callback_url = format!("trae://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 Trae 回调 URL:");
    println!("    {}", callback_url);

    // 步骤 5: 保存到本地存储
    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    // 保存访问令牌
    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    // 保存 API 提供商
    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");

    // 步骤 6: 打开 Trae 回调 URL
    println!("[*] 正在打开 Trae...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 Trae 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Trae 失败。请确保 Trae 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Trae 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Trae 失败。请确保 Trae 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Trae 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Trae 失败。请确保 Trae 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] Trae 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

#[tauri::command]
pub async fn login_to_qoder(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 Qoder ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    let callback_url = format!("qoder://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 Qoder 回调 URL:");
    println!("    {}", callback_url);

    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");
    println!("[*] 正在打开 Qoder...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 Qoder 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Qoder 失败。请确保 Qoder 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Qoder 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Qoder 失败。请确保 Qoder 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Qoder 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Qoder 失败。请确保 Qoder 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] Qoder 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

#[tauri::command]
pub async fn login_to_kiro(token: String) -> Result<LoginToVSCodeResponse, String> {
    println!("\n[*] ========== 一键登录到 Kiro ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    let pkce = PkceParams::generate();
    println!("[*] 生成 PKCE 参数:");
    println!("    State: {}", pkce.state);
    println!("    Code Verifier: {}", pkce.code_verifier);
    println!("    Code Challenge: {}", pkce.code_challenge);

    let api = VerdentApi::new();
    let auth_code = match api.request_auth_code(&token, &pkce).await {
        Ok(code) => {
            println!("[✓] 授权码获取成功: {}", code);
            code
        },
        Err(e) => {
            eprintln!("[×] 获取授权码失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("获取授权码失败: {}。请检查 Token 是否有效或已过期。", e)),
            });
        }
    };

    let access_token: String = match api.exchange_token(&auth_code, &pkce.code_verifier).await {
        Ok(token) => {
            println!("[✓] 访问令牌获取成功");
            println!("    Token 长度: {} 字符", token.len());
            token
        },
        Err(e) => {
            eprintln!("[×] 交换访问令牌失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("交换访问令牌失败: {}", e)),
            });
        }
    };

    let callback_url = format!("kiro://verdentai.verdent/auth?code={}&state={}", auth_code, pkce.state);
    println!("[*] 构建 Kiro 回调 URL:");
    println!("    {}", callback_url);

    use crate::storage::StorageManager;
    let storage = match StorageManager::new() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[×] 初始化存储管理器失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("初始化存储失败: {}", e)),
            });
        }
    };

    if let Err(e) = storage.save("secrets_ycAuthToken", serde_json::json!({
        "value": access_token
    })) {
        eprintln!("[×] 保存访问令牌失败: {}", e);
        return Ok(LoginToVSCodeResponse {
            success: false,
            error: Some(format!("保存访问令牌失败: {}", e)),
        });
    }

    if let Err(e) = storage.save("globalState_apiProvider", serde_json::json!({
        "value": "verdent"
    })) {
        eprintln!("[×] 保存 API 提供商失败: {}", e);
    }

    println!("[✓] 访问令牌已保存到本地存储");
    println!("[*] 正在打开 Kiro...");
    println!("    回调 URL: {}", callback_url);

    #[cfg(target_os = "windows")]
    {
        println!("    使用 rundll32 打开 URL...");
        let result = Command::new("rundll32")
            .args(&["url.dll,FileProtocolHandler", &callback_url])
            .spawn();

        if let Err(e) = result {
            eprintln!("[×] 打开 Kiro 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Kiro 失败。请确保 Kiro 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }

        println!("[✓] 已使用 rundll32 打开 URL");
    }

    #[cfg(target_os = "macos")]
    {
        println!("    使用 macOS 命令打开...");
        if let Err(e) = Command::new("open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Kiro 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Kiro 失败。请确保 Kiro 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    #[cfg(target_os = "linux")]
    {
        println!("    使用 Linux 命令打开...");
        if let Err(e) = Command::new("xdg-open").arg(&callback_url).spawn() {
            eprintln!("[×] 打开 Kiro 失败: {}", e);
            return Ok(LoginToVSCodeResponse {
                success: false,
                error: Some(format!("打开 Kiro 失败。请确保 Kiro 已安装并且 Verdent 插件已启用。错误: {}", e)),
            });
        }
    }

    println!("[✓] Kiro 回调链接已发送");
    println!("[✓] 登录流程完成!");
    println!("======================================\n");

    Ok(LoginToVSCodeResponse {
        success: true,
        error: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshAccountResponse {
    pub success: bool,
    pub account: Option<Account>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn refresh_account_info(account_id: String) -> Result<RefreshAccountResponse, String> {
    println!("\n[*] ========== 刷新账户信息 ==========");
    println!("[*] 账户 ID: {}", account_id);

    // 加载账户管理器
    let manager = AccountManager::new().map_err(|e| {
        eprintln!("[×] 加载账户管理器失败: {}", e);
        format!("加载账户管理器失败: {}", e)
    })?;

    // 查找账户
    let account = manager.get_account(&account_id).map_err(|e| {
        eprintln!("[×] 账户不存在: {}", account_id);
        format!("账户不存在: {}", e)
    })?;

    // 检查是否有 Token
    let token = account.token.as_ref().ok_or_else(|| {
        eprintln!("[×] 账户没有 Token,无法刷新");
        "账户没有 Token,无法刷新".to_string()
    })?;

    println!("[*] 使用 Token 获取用户信息 (带重试机制)...");

    // 调用 API 获取用户信息 (最多重试 3 次)
    let api = VerdentApi::new();
    let user_info = api.fetch_user_info_with_retry(token, 3).await.map_err(|e| {
        eprintln!("[×] 获取用户信息失败: {}", e);
        e
    })?;

    println!("[✓] 成功获取用户信息");

    // 计算额度信息
    let quota_consumed = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_consumed)
        .unwrap_or(0.0);
    let quota_free = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_free)
        .unwrap_or(0.0);
    let quota_remaining = quota_free - quota_consumed;

    println!("[*] 额度信息:");
    println!("    总额度: {:.2}", quota_free);
    println!("    已消耗: {:.2}", quota_consumed);
    println!("    剩余额度: {:.2}", quota_remaining);

    // 打印过期时间信息用于调试
    println!("[*] 过期时间信息:");
    println!("    API返回的expire_time: {:?}", user_info.expire_time);
    println!("    订阅类型: {:?}", user_info.subscription_type);
    println!("    试用天数: {:?}", user_info.trial_days);

    // 打印订阅信息
    if let Some(ref sub_info) = user_info.subscription_info {
        println!("[*] 订阅信息:");
        println!("    订阅级别: {:?}", sub_info.level_name);
        println!("    订阅周期结束时间(时间戳): {:?}", sub_info.current_period_end);
        println!("    自动续订: {:?}", sub_info.auto_renew);
    } else {
        println!("[!] API未返回订阅信息");
    }

    // 更新账户信息
    let mut updated_account = account.clone();
    updated_account.quota_total = Some(format!("{:.2}", quota_free));
    updated_account.quota_used = Some(format!("{:.2}", quota_consumed));
    updated_account.quota_remaining = Some(format!("{:.2}", quota_remaining));

    // 订阅类型 - 优先使用 planName
    updated_account.subscription_type = user_info.subscription_info.as_ref()
        .and_then(|si| si.plan_name.clone())
        .or_else(|| user_info.subscription_info.as_ref()
            .and_then(|si| si.level_name.clone()))
        .or(user_info.subscription_type);

    updated_account.trial_days = user_info.trial_days;

    // 从subscriptionInfo中提取订阅周期结束时间和自动续订状态
    let current_period_end = user_info.subscription_info
        .as_ref()
        .and_then(|si| si.current_period_end);

    updated_account.current_period_end = current_period_end;
    updated_account.auto_renew = user_info.subscription_info
        .as_ref()
        .and_then(|si| si.auto_renew);

    // 计算订阅到期时间: 从 currentPeriodEnd 转换
    updated_account.expire_time = if let Some(period_end) = current_period_end {
        // 将 Unix 时间戳转换为 RFC3339 格式
        let dt = chrono::DateTime::from_timestamp(period_end, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        Some(dt.to_rfc3339())
    } else {
        None
    };

    // 从 JWT Token 中提取 Token 过期时间
    updated_account.token_expire_time = jwt_utils::extract_token_expire_time(token);

    updated_account.last_updated = Some(chrono::Local::now().to_rfc3339());

    // 保存更新后的账户
    manager.update_account(&updated_account.id, updated_account.clone()).map_err(|e| {
        eprintln!("[×] 保存账户失败: {}", e);
        format!("保存账户失败: {}", e)
    })?;

    println!("[✓] 账户信息已更新并保存");
    println!("======================================\n");

    Ok(RefreshAccountResponse {
        success: true,
        account: Some(updated_account),
        error: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestLoginResponse {
    pub success: bool,
    pub token: Option<String>,
    pub account: Option<Account>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn test_login_and_update(account_id: String) -> Result<TestLoginResponse, String> {
    println!("\n[*] ========== 测试登录并更新 Token ==========");
    println!("[*] 账户 ID: {}", account_id);

    // 加载账户管理器
    let manager = AccountManager::new().map_err(|e| {
        eprintln!("[×] 加载账户管理器失败: {}", e);
        format!("加载账户管理器失败: {}", e)
    })?;

    // 查找账户
    let account = manager.get_account(&account_id).map_err(|e| {
        eprintln!("[×] 账户不存在: {}", account_id);
        format!("账户不存在: {}", e)
    })?;

    println!("[*] 测试账户: {}", account.email);
    println!("[*] 使用邮箱和密码登录...");

    // 调用登录 API
    let api = VerdentApi::new();
    let login_result = api.login(&account.email, &account.password).await.map_err(|e| {
        eprintln!("[×] 登录失败: {}", e);
        format!("登录失败: {}", e)
    })?;

    println!("[✓] 登录成功!");
    println!("[*] Token: {}...", &login_result.token[..20]);

    // 更新账户的 Token
    let mut updated_account = account.clone();
    updated_account.token = Some(login_result.token.clone());
    updated_account.last_updated = Some(chrono::Local::now().to_rfc3339());

    // 如果登录响应中有过期时间,也更新它
    if let Some(expire_timestamp) = login_result.expire_time {
        let expire_datetime = chrono::DateTime::from_timestamp(expire_timestamp, 0)
            .map(|dt| dt.to_rfc3339());
        if let Some(expire_time) = expire_datetime {
            updated_account.expire_time = Some(expire_time);
            println!("[*] 更新过期时间: {}", updated_account.expire_time.as_ref().unwrap());
        }
    }

    // 保存更新后的账户
    manager.update_account(&updated_account.id, updated_account.clone()).map_err(|e| {
        eprintln!("[×] 保存账户失败: {}", e);
        format!("保存账户失败: {}", e)
    })?;

    println!("[✓] Token 已更新并保存");

    // 尝试获取用户信息以更新额度等信息 (带重试)
    println!("[*] 获取用户信息 (带重试机制)...");
    match api.fetch_user_info_with_retry(&login_result.token, 3).await {
        Ok(user_info) => {
            println!("[✓] 成功获取用户信息");

            // 计算额度信息
            let quota_consumed = user_info.token_info.as_ref()
                .and_then(|ti| ti.token_consumed)
                .unwrap_or(0.0);
            let quota_free = user_info.token_info.as_ref()
                .and_then(|ti| ti.token_free)
                .unwrap_or(0.0);
            let quota_remaining = quota_free - quota_consumed;

            // 更新账户信息
            updated_account.quota_total = Some(format!("{:.2}", quota_free));
            updated_account.quota_used = Some(format!("{:.2}", quota_consumed));
            updated_account.quota_remaining = Some(format!("{:.2}", quota_remaining));

            // 订阅类型 - 优先使用 planName
            updated_account.subscription_type = user_info.subscription_info.as_ref()
                .and_then(|si| si.plan_name.clone())
                .or_else(|| user_info.subscription_info.as_ref()
                    .and_then(|si| si.level_name.clone()))
                .or(user_info.subscription_type);

            updated_account.trial_days = user_info.trial_days;

            // 从subscriptionInfo中提取订阅周期结束时间和自动续订状态
            let current_period_end = user_info.subscription_info
                .as_ref()
                .and_then(|si| si.current_period_end);

            updated_account.current_period_end = current_period_end;
            updated_account.auto_renew = user_info.subscription_info
                .as_ref()
                .and_then(|si| si.auto_renew);

            // 计算订阅到期时间: 从 currentPeriodEnd 转换
            updated_account.expire_time = if let Some(period_end) = current_period_end {
                // 将 Unix 时间戳转换为 RFC3339 格式
                let dt = chrono::DateTime::from_timestamp(period_end, 0)
                    .unwrap_or_else(|| chrono::Utc::now());
                Some(dt.to_rfc3339())
            } else {
                None
            };

            // 从 JWT Token 中提取 Token 过期时间
            updated_account.token_expire_time = jwt_utils::extract_token_expire_time(&login_result.token);

            // 再次保存
            manager.update_account(&updated_account.id, updated_account.clone()).map_err(|e| {
                eprintln!("[×] 保存账户失败: {}", e);
                format!("保存账户失败: {}", e)
            })?;

            println!("[✓] 账户信息已更新");
        }
        Err(e) => {
            eprintln!("[!] 获取用户信息失败: {}", e);
            println!("[!] Token 已保存,但未能获取额度信息");
        }
    }

    println!("======================================\n");

    Ok(TestLoginResponse {
        success: true,
        token: Some(login_result.token),
        account: Some(updated_account),
        error: None,
    })
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportAccountResponse {
    pub success: bool,
    pub account: Option<Account>,
    pub error: Option<String>,
}

#[tauri::command]
pub async fn import_account_by_token(token: String) -> Result<ImportAccountResponse, String> {
    println!("\n[*] ========== 导入 Token 账户 ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    // 验证 Token 格式 (基本检查)
    if token.trim().is_empty() {
        eprintln!("[×] Token 为空");
        return Ok(ImportAccountResponse {
            success: false,
            account: None,
            error: Some("Token 不能为空".to_string()),
        });
    }

    let token = token.trim().to_string();

    // 调用 API 获取用户信息 (带重试)
    println!("[*] 使用 Token 获取用户信息 (带重试机制)...");
    let api = VerdentApi::new();
    let user_info = match api.fetch_user_info_with_retry(&token, 3).await {
        Ok(info) => {
            println!("[✓] 成功获取用户信息");
            info
        }
        Err(e) => {
            eprintln!("[×] 获取用户信息失败: {}", e);
            return Ok(ImportAccountResponse {
                success: false,
                account: None,
                error: Some(e),
            });
        }
    };

    // 提取邮箱
    let email: String = match user_info.email {
        Some(e) => e,
        None => {
            eprintln!("[×] API 响应中缺少邮箱信息");
            return Ok(ImportAccountResponse {
                success: false,
                account: None,
                error: Some("API 响应中缺少邮箱信息".to_string()),
            });
        }
    };

    println!("[*] 账户邮箱: {}", email);

    // 计算额度信息
    let quota_consumed = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_consumed)
        .unwrap_or(0.0);
    let quota_free = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_free)
        .unwrap_or(0.0);
    let quota_remaining = quota_free - quota_consumed;

    println!("[*] 额度信息:");
    println!("    已消耗: {:.2}", quota_consumed);
    println!("    总额度: {:.2}", quota_free);
    println!("    剩余: {:.2}", quota_remaining);

    // 提取订阅信息 - 优先使用 planName
    let subscription_type = user_info.subscription_info.as_ref()
        .and_then(|si| si.plan_name.clone())
        .or_else(|| user_info.subscription_info.as_ref()
            .and_then(|si| si.level_name.clone()))
        .or(user_info.subscription_type.clone());

    let current_period_end = user_info.subscription_info.as_ref()
        .and_then(|si| si.current_period_end);

    let auto_renew = user_info.subscription_info.as_ref()
        .and_then(|si| si.auto_renew)
        .unwrap_or(false);

    let trial_days = user_info.trial_days;

    // 计算订阅到期时间: 从 currentPeriodEnd 转换
    let expire_time = if let Some(period_end) = current_period_end {
        // 将 Unix 时间戳转换为 RFC3339 格式
        let dt = chrono::DateTime::from_timestamp(period_end, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        Some(dt.to_rfc3339())
    } else {
        None
    };

    // 从 JWT Token 中提取 Token 过期时间
    let token_expire_time = jwt_utils::extract_token_expire_time(&token);

    println!("[*] 订阅信息:");
    println!("    类型: {:?}", subscription_type);
    println!("    周期结束时间戳: {:?}", current_period_end);
    println!("    订阅到期时间: {:?}", expire_time);
    println!("    自动续订: {}", auto_renew);
    println!("    试用天数: {:?}", trial_days);
    println!("    Token 过期时间: {:?}", token_expire_time);

    // 加载账户管理器
    let manager = AccountManager::new().map_err(|e| {
        eprintln!("[×] 加载账户管理器失败: {}", e);
        format!("加载账户管理器失败: {}", e)
    })?;

    // 检查账户是否已存在
    let existing_account = manager.get_all_accounts()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|acc| acc.email == email);

    let account = if let Some(mut existing) = existing_account {
        println!("[*] 账户已存在,更新信息...");

        // 更新现有账户
        existing.token = Some(token.clone());
        existing.quota_remaining = Some(format!("{:.2}", quota_remaining));
        existing.quota_used = Some(format!("{:.2}", quota_consumed));
        existing.quota_total = Some(format!("{:.2}", quota_free));
        existing.subscription_type = subscription_type;
        existing.trial_days = trial_days;
        existing.current_period_end = current_period_end;
        existing.auto_renew = Some(auto_renew);
        existing.expire_time = expire_time;
        existing.token_expire_time = token_expire_time.clone();
        existing.last_updated = Some(chrono::Local::now().to_rfc3339());

        // 更新状态
        if let Some(exp_time) = &existing.expire_time {
            if let Ok(expire_dt) = chrono::DateTime::parse_from_rfc3339(exp_time) {
                let now = chrono::Local::now();
                existing.status = if expire_dt > now {
                    Some("active".to_string())
                } else {
                    Some("expired".to_string())
                };
            }
        }

        manager.update_account(&existing.id, existing.clone()).map_err(|e| {
            eprintln!("[×] 更新账户失败: {}", e);
            format!("更新账户失败: {}", e)
        })?;

        println!("[✓] 账户信息已更新");
        existing
    } else {
        println!("[*] 创建新账户...");

        // 创建新账户
        let new_account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.clone(),
            password: String::new(), // 通过 Token 导入的账户没有密码
            register_time: Some(chrono::Local::now().to_rfc3339()),
            expire_time,
            status: Some("active".to_string()),
            token: Some(token.clone()),
            quota_remaining: Some(format!("{:.2}", quota_remaining)),
            quota_used: Some(format!("{:.2}", quota_consumed)),
            quota_total: Some(format!("{:.2}", quota_free)),
            subscription_type,
            trial_days,
            current_period_end,
            auto_renew: Some(auto_renew),
            token_expire_time: token_expire_time.clone(),
            last_updated: Some(chrono::Local::now().to_rfc3339()),
        };

        manager.add_account(new_account.clone()).map_err(|e| {
            eprintln!("[×] 添加账户失败: {}", e);
            format!("添加账户失败: {}", e)
        })?;

        println!("[✓] 新账户已创建");
        new_account
    };

    println!("[✓] Token 导入成功: {}", email);
    println!("======================================\n");

    Ok(ImportAccountResponse {
        success: true,
        account: Some(account),
        error: None,
    })
}

/// 通过账号密码导入账户
/// 会自动登录获取Token并保存账户信息
#[tauri::command]
pub async fn import_account_by_credentials(email: String, password: String) -> Result<ImportAccountResponse, String> {
    println!("\n[*] ========== 通过账号密码导入 ==========");
    println!("[*] 邮箱: {}", email);
    println!("[*] 密码: {}", "*".repeat(password.len()));

    // 验证输入
    if email.trim().is_empty() || password.trim().is_empty() {
        eprintln!("[×] 邮箱或密码为空");
        return Ok(ImportAccountResponse {
            success: false,
            account: None,
            error: Some("邮箱和密码不能为空".to_string()),
        });
    }

    let email = email.trim().to_string();
    let password = password.trim().to_string();

    // 步骤1: 尝试登录获取Token
    println!("[*] 正在登录账户...");
    let api = VerdentApi::new();

    let token = match api.login(&email, &password).await {
        Ok(login_data) => {
            println!("[✓] 登录成功，获取到 Token");
            login_data.token
        },
        Err(e) => {
            eprintln!("[×] 登录失败: {}", e);
            return Ok(ImportAccountResponse {
                success: false,
                account: None,
                error: Some(format!("登录失败: {}", e)),
            });
        }
    };

    println!("[*] Token 长度: {} 字符", token.len());

    // 步骤2: 获取用户信息
    println!("[*] 获取账户详细信息...");
    let user_info = match api.fetch_user_info_with_retry(&token, 3).await {
        Ok(info) => {
            println!("[✓] 成功获取用户信息");
            info
        },
        Err(e) => {
            eprintln!("[×] 获取用户信息失败: {}", e);
            // 即使获取用户信息失败，也保存基本账户信息
            println!("[!] 将保存基本账户信息");

            // 创建基本账户
            let manager = AccountManager::new().map_err(|e| format!("加载账户管理器失败: {}", e))?;

            // 检查账户是否已存在
            let existing_account = manager.get_all_accounts()
                .map_err(|e| e.to_string())?
                .into_iter()
                .find(|acc| acc.email == email);

            let account = if let Some(mut existing) = existing_account {
                // 更新现有账户
                existing.password = password.clone();
                existing.token = Some(token.clone());
                existing.last_updated = Some(chrono::Local::now().to_rfc3339());
                existing.status = Some("active".to_string());

                manager.update_account(&existing.id, existing.clone())
                    .map_err(|e| format!("更新账户失败: {}", e))?;

                existing
            } else {
                // 创建新账户
                let new_account = Account {
                    id: uuid::Uuid::new_v4().to_string(),
                    email: email.clone(),
                    password: password.clone(),
                    register_time: Some(chrono::Local::now().to_rfc3339()),
                    expire_time: None,
                    status: Some("active".to_string()),
                    token: Some(token.clone()),
                    quota_remaining: None,
                    quota_used: None,
                    quota_total: None,
                    subscription_type: None,
                    trial_days: None,
                    current_period_end: None,
                    auto_renew: None,
                    token_expire_time: jwt_utils::extract_token_expire_time(&token),
                    last_updated: Some(chrono::Local::now().to_rfc3339()),
                };

                manager.add_account(new_account.clone())
                    .map_err(|e| format!("添加账户失败: {}", e))?;

                new_account
            };

            println!("[✓] 账户导入成功（基本信息）: {}", email);
            return Ok(ImportAccountResponse {
                success: true,
                account: Some(account),
                error: None,
            });
        }
    };

    // 步骤3: 提取并保存完整信息
    // 计算额度信息
    let quota_consumed = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_consumed)
        .unwrap_or(0.0);
    let quota_free = user_info.token_info.as_ref()
        .and_then(|ti| ti.token_free)
        .unwrap_or(0.0);
    let quota_remaining = quota_free - quota_consumed;

    println!("[*] 额度信息:");
    println!("    已消耗: {:.2}", quota_consumed);
    println!("    总额度: {:.2}", quota_free);
    println!("    剩余: {:.2}", quota_remaining);

    // 提取订阅信息
    let subscription_type = user_info.subscription_info.as_ref()
        .and_then(|si| si.plan_name.clone())
        .or_else(|| user_info.subscription_info.as_ref()
            .and_then(|si| si.level_name.clone()))
        .or(user_info.subscription_type.clone());

    let current_period_end = user_info.subscription_info.as_ref()
        .and_then(|si| si.current_period_end);

    let auto_renew = user_info.subscription_info.as_ref()
        .and_then(|si| si.auto_renew)
        .unwrap_or(false);

    let trial_days = user_info.trial_days;

    // 计算订阅到期时间
    let expire_time = if let Some(period_end) = current_period_end {
        let dt = chrono::DateTime::from_timestamp(period_end, 0)
            .unwrap_or_else(|| chrono::Utc::now());
        Some(dt.to_rfc3339())
    } else {
        None
    };

    // 从JWT Token 中提取 Token 过期时间
    let token_expire_time = jwt_utils::extract_token_expire_time(&token);

    // 步骤4: 保存到账户管理器
    let manager = AccountManager::new().map_err(|e| format!("加载账户管理器失败: {}", e))?;

    // 检查账户是否已存在
    let existing_account = manager.get_all_accounts()
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|acc| acc.email == email);

    let account = if let Some(mut existing) = existing_account {
        println!("[*] 账户已存在，更新信息...");

        // 更新现有账户
        existing.password = password.clone();
        existing.token = Some(token.clone());
        existing.quota_remaining = Some(format!("{:.2}", quota_remaining));
        existing.quota_used = Some(format!("{:.2}", quota_consumed));
        existing.quota_total = Some(format!("{:.2}", quota_free));
        existing.subscription_type = subscription_type;
        existing.trial_days = trial_days;
        existing.current_period_end = current_period_end;
        existing.auto_renew = Some(auto_renew);
        existing.expire_time = expire_time;
        existing.token_expire_time = token_expire_time.clone();
        existing.last_updated = Some(chrono::Local::now().to_rfc3339());

        // 更新状态
        if let Some(exp_time) = &existing.expire_time {
            if let Ok(expire_dt) = chrono::DateTime::parse_from_rfc3339(exp_time) {
                let now = chrono::Local::now();
                existing.status = if expire_dt > now {
                    Some("active".to_string())
                } else {
                    Some("expired".to_string())
                };
            }
        }

        manager.update_account(&existing.id, existing.clone())
            .map_err(|e| format!("更新账户失败: {}", e))?;

        println!("[✓] 账户信息已更新");
        existing
    } else {
        println!("[*] 创建新账户...");

        // 创建新账户
        let new_account = Account {
            id: uuid::Uuid::new_v4().to_string(),
            email: email.clone(),
            password: password.clone(),
            register_time: Some(chrono::Local::now().to_rfc3339()),
            expire_time,
            status: Some("active".to_string()),
            token: Some(token.clone()),
            quota_remaining: Some(format!("{:.2}", quota_remaining)),
            quota_used: Some(format!("{:.2}", quota_consumed)),
            quota_total: Some(format!("{:.2}", quota_free)),
            subscription_type,
            trial_days,
            current_period_end,
            auto_renew: Some(auto_renew),
            token_expire_time: token_expire_time.clone(),
            last_updated: Some(chrono::Local::now().to_rfc3339()),
        };

        manager.add_account(new_account.clone())
            .map_err(|e| format!("添加账户失败: {}", e))?;

        println!("[✓] 新账户已创建");
        new_account
    };

    println!("[✓] 账号密码导入成功: {}", email);
    println!("======================================\n");

    Ok(ImportAccountResponse {
        success: true,
        account: Some(account),
        error: None,
    })
}

// ===== 用户设置相关命令 =====

#[tauri::command]
pub async fn get_user_settings() -> Result<UserSettings, String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.load().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_user_settings(settings: UserSettings) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.save(&settings).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_current_account(account_id: Option<String>) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.update_current_account(account_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_email_privacy(privacy_mode: bool) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.update_email_privacy(privacy_mode).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_selected_accounts(selected: Vec<String>) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.update_selected_accounts(selected).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_filter_type(filter_type: Option<String>) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.update_filter_type(filter_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_register_config(
    count: i32,
    max_workers: i32,
    password: String,
    use_random_password: bool,
    headless: bool
) -> Result<(), String> {
    let manager = SettingsManager::new().map_err(|e| e.to_string())?;
    manager.update_register_config(count, max_workers, password, use_random_password, headless)
        .map_err(|e| e.to_string())
}

/// 打开存储文件夹
#[tauri::command]
pub fn open_storage_folder() -> Result<String, String> {
    let storage_dir = dirs::home_dir()
        .ok_or_else(|| "无法获取主目录".to_string())?
        .join(".verdent_accounts");

    // 确保目录存在
    if !storage_dir.exists() {
        std::fs::create_dir_all(&storage_dir)
            .map_err(|e| format!("创建存储目录失败: {}", e))?;
    }

    let path_str = storage_dir.to_string_lossy().to_string();

    // 根据操作系统打开文件管理器
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("打开资源管理器失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&path_str)
            .spawn()
            .map_err(|e| format!("打开 Finder 失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // 尝试不同的文件管理器
        let managers = ["xdg-open", "gnome-open", "nautilus", "dolphin", "thunar", "pcmanfm"];
        let mut opened = false;

        for manager in &managers {
            if Command::new(manager)
                .arg(&path_str)
                .spawn()
                .is_ok() {
                opened = true;
                break;
            }
        }

        if !opened {
            return Err("无法打开文件管理器，请手动访问: ".to_string() + &path_str);
        }
    }

    Ok(path_str)
}

/// 获取代理设置
#[tauri::command]
pub async fn get_proxy_settings() -> Result<crate::proxy_manager::ProxySettings, String> {
    crate::proxy_manager::ProxyManager::load()
}

/// 保存代理设置
#[tauri::command]
pub async fn save_proxy_settings(enabled: bool, url: String) -> Result<(), String> {
    let settings = crate::proxy_manager::ProxySettings { enabled, url };
    crate::proxy_manager::ProxyManager::save(&settings)
}

/// 获取试用绑卡链接
#[tauri::command]
pub async fn get_trial_checkout_url(token: String) -> Result<serde_json::Value, String> {
    use reqwest::header::{HeaderMap, HeaderValue, COOKIE};
    use serde_json::json;

    // 加载代理设置
    let proxy_settings = crate::proxy_manager::ProxyManager::load()
        .unwrap_or_default();

    // 创建HTTP客户端
    let mut client_builder = reqwest::Client::builder();

    if proxy_settings.enabled && !proxy_settings.url.is_empty() {
        println!("[DEBUG] 使用代理: {}", proxy_settings.url);
        let proxy = reqwest::Proxy::all(&proxy_settings.url)
            .map_err(|e| format!("代理配置错误: {}", e))?;
        client_builder = client_builder.proxy(proxy);
    }

    let client = client_builder
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 构建请求头
    let mut headers = HeaderMap::new();
    let cookie_value = format!("token={}", token);
    headers.insert(COOKIE, HeaderValue::from_str(&cookie_value)
        .map_err(|e| format!("设置Cookie失败: {}", e))?);

    // 获取用户信息
    let user_info_url = "https://agent.verdent.ai/user/center/info";
    let user_info_response = client
        .get(user_info_url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?;

    if !user_info_response.status().is_success() {
        return Err(format!("获取用户信息失败: HTTP {}", user_info_response.status()));
    }

    let user_info: serde_json::Value = user_info_response.json()
        .await
        .map_err(|e| format!("解析用户信息失败: {}", e))?;

    // 从用户信息中获取 trialPlanId
    let trial_plan_id = user_info["data"]["trialPlanId"]
        .as_str()
        .ok_or_else(|| "无法获取试用计划ID".to_string())?;

    // 生成设备ID (UUID v4)
    let device_id = uuid::Uuid::new_v4().to_string();

    // 创建订阅请求
    let create_subscription_url = "https://api.verdent.ai/verdent/subscription/create";
    let subscription_body = json!({
        "plan_id": trial_plan_id,
        "device_id": device_id,
        "source": "verdent"
    });

    let subscription_response = client
        .post(create_subscription_url)
        .headers(headers)
        .json(&subscription_body)
        .send()
        .await
        .map_err(|e| format!("创建订阅失败: {}", e))?;

    if !subscription_response.status().is_success() {
        return Err(format!("创建订阅失败: HTTP {}", subscription_response.status()));
    }

    let subscription_result: serde_json::Value = subscription_response.json()
        .await
        .map_err(|e| format!("解析订阅结果失败: {}", e))?;

    // 检查是否有错误码
    if let Some(err_code) = subscription_result["errCode"].as_i64() {
        if err_code != 0 {
            let err_msg = subscription_result["errMsg"]
                .as_str()
                .unwrap_or("未知错误");
            return Err(format!("API错误: {}", err_msg));
        }
    }

    // 获取 checkout_url
    let checkout_url = subscription_result["data"]["checkout_url"]
        .as_str()
        .ok_or_else(|| "无法获取绑卡链接".to_string())?;

    Ok(json!({
        "success": true,
        "checkout_url": checkout_url
    }))
}

/// 打开无痕浏览器
#[tauri::command]
pub async fn open_incognito_browser(url: String) -> Result<(), String> {
    use std::process::Command;

    #[cfg(target_os = "windows")]
    {
        // Windows: 使用Chrome的无痕模式
        let browsers = vec![
            ("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe", "--incognito"),
            ("C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe", "--incognito"),
            ("C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe", "--inprivate"),
            ("C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe", "--inprivate"),
        ];

        for (browser_path, flag) in browsers {
            if std::path::Path::new(browser_path).exists() {
                Command::new(browser_path)
                    .arg(flag)
                    .arg(&url)
                    .spawn()
                    .map_err(|e| format!("打开浏览器失败: {}", e))?;
                return Ok(());
            }
        }

        // 如果找不到特定浏览器，使用系统默认浏览器
        Command::new("cmd")
            .args(&["/C", "start", "", &url])
            .spawn()
            .map_err(|e| format!("打开默认浏览器失败: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: 使用Safari的私密浏览模式
        Command::new("osascript")
            .arg("-e")
            .arg(&format!(
                "tell application \"Safari\" to open location \"{}\" in a new private window",
                url
            ))
            .spawn()
            .map_err(|e| format!("打开Safari私密浏览失败: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: 尝试不同的浏览器
        let browsers = vec![
            ("google-chrome", "--incognito"),
            ("chromium", "--incognito"),
            ("firefox", "--private-window"),
        ];

        for (browser, flag) in browsers {
            if Command::new("which").arg(browser).output().is_ok() {
                Command::new(browser)
                    .arg(flag)
                    .arg(&url)
                    .spawn()
                    .map_err(|e| format!("打开浏览器失败: {}", e))?;
                return Ok(());
            }
        }

        // 使用xdg-open作为后备选项
        Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map_err(|e| format!("打开默认浏览器失败: {}", e))?;
    }

    Ok(())
}

/// Verdent 客户端登录响应
#[derive(Debug, Serialize, Deserialize)]
pub struct VerdentClientLoginResponse {
    pub success: bool,
    pub error: Option<String>,
    pub new_machine_guid: Option<String>,
}

/// 一键登录到 Verdent 客户端
/// 使用账户的 token 直接登录到本地 Verdent 客户端
///
/// 功能包括:
/// 1. 将 Token 写入 Verdent 客户端配置 (Windows Credential Manager / macOS Keychain / Linux 配置文件)
/// 2. 重置机器码 (仅 Windows)
/// 3. 重启 Verdent 客户端
#[tauri::command]
pub async fn login_to_verdent_client(token: String) -> Result<VerdentClientLoginResponse, String> {
    println!("\n[*] ========== 一键登录到 Verdent 客户端 ==========");
    println!("[*] Token 长度: {} 字符", token.len());

    let mut manager = VerdentClientManager::new();

    // 获取用户自定义的 Verdent.exe 路径
    let custom_path = match SettingsManager::new() {
        Ok(settings_mgr) => {
            match settings_mgr.load() {
                Ok(settings) => {
                    println!("[*] 从配置文件读取到的路径: {:?}", settings.verdent_exe_path);
                    settings.verdent_exe_path
                },
                Err(e) => {
                    println!("[!] 加载配置文件失败: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("[!] 初始化设置管理器失败: {}", e);
            None
        }
    };

    println!("[*] 将使用的自定义路径: {:?}", custom_path);

    // 步骤 1: 检查 Verdent 客户端是否安装
    println!("[*] 检查 Verdent 客户端安装...");
    if let Err(e) = manager.find_verdent_path(custom_path.as_deref()) {
        eprintln!("[×] 未找到 Verdent 客户端: {}", e);
        return Ok(VerdentClientLoginResponse {
            success: false,
            error: Some(format!("未找到 Verdent 客户端。请确保已安装 Verdent 桌面应用，或者设置 Verdent.exe 的正确位置。\n错误: {}", e)),
            new_machine_guid: None,
        });
    }

    // 步骤 2: 解析 Token 获取过期时间
    println!("[*] 解析 Token...");
    let expire_at = match jwt_utils::parse_jwt_payload(&token) {
        Ok(payload) => {
            if let Some(exp) = payload.exp {
                // JWT exp 是秒级时间戳,需要转换为毫秒
                exp * 1000
            } else {
                // 如果没有过期时间,默认设置为 30 天后
                chrono::Utc::now().timestamp_millis() + (30 * 24 * 60 * 60 * 1000)
            }
        },
        Err(e) => {
            eprintln!("[!] 解析 Token 失败,使用默认过期时间: {}", e);
            // 默认 30 天后过期
            chrono::Utc::now().timestamp_millis() + (30 * 24 * 60 * 60 * 1000)
        }
    };

    println!("[*] Token 过期时间: {}", expire_at);

    // 步骤 3: 写入 Token 到客户端配置
    println!("[*] 写入 Token 到 Verdent 客户端配置...");
    if let Err(e) = manager.write_token(&token, expire_at) {
        eprintln!("[×] 写入 Token 失败: {}", e);

        // 检查是否是权限问题
        let error_msg = if e.to_string().contains("需要管理员权限") || e.to_string().contains("Permission") {
            "写入凭证失败:需要管理员权限。\n\n请以管理员身份运行本程序。".to_string()
        } else {
            format!("写入 Token 失败: {}", e)
        };

        return Ok(VerdentClientLoginResponse {
            success: false,
            error: Some(error_msg),
            new_machine_guid: None,
        });
    }

    // 步骤 4: 重置机器码 (仅 Windows)
    let new_machine_guid = if cfg!(target_os = "windows") {
        println!("[*] 重置机器码...");
        match manager.reset_machine_guid() {
            Ok(guid) => {
                println!("[✓] 机器码已重置: {}", guid);
                Some(guid)
            },
            Err(e) => {
                eprintln!("[!] 重置机器码失败: {}", e);

                // 机器码重置失败不影响登录,只是警告
                if e.to_string().contains("需要管理员权限") || e.to_string().contains("Permission") {
                    println!("[!] 跳过机器码重置 (需要管理员权限)");
                } else {
                    println!("[!] 跳过机器码重置: {}", e);
                }
                None
            }
        }
    } else {
        None
    };

    // 步骤 5: 重启 Verdent 客户端
    println!("[*] 重启 Verdent 客户端...");
    if let Err(e) = manager.restart_verdent() {
        eprintln!("[×] 重启 Verdent 失败: {}", e);
        return Ok(VerdentClientLoginResponse {
            success: false,
            error: Some(format!("Token 已写入,但重启 Verdent 失败: {}\n\n请手动重启 Verdent 客户端。", e)),
            new_machine_guid,
        });
    }

    println!("[✓] Verdent 客户端登录完成!");
    println!("======================================\n");

    Ok(VerdentClientLoginResponse {
        success: true,
        error: None,
        new_machine_guid,
    })
}


// ============================================================================
// 管理员权限管理命令
// ============================================================================

/// 检查是否有管理员权限
#[tauri::command]
pub async fn check_admin_privileges() -> Result<bool, String> {
    Ok(crate::admin_helper::is_running_as_admin())
}

/// 请求管理员权限并重启应用
#[tauri::command]
pub async fn request_admin_privileges() -> Result<(), String> {
    crate::admin_helper::restart_as_admin()
}

/// 创建开机自启动任务（以管理员权限）
#[tauri::command]
pub async fn setup_admin_autostart() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if !crate::admin_helper::is_running_as_admin() {
            return Err("需要管理员权限来创建任务计划".to_string());
        }
        
        crate::admin_helper::create_admin_task("VerdentAccountManager")
            .map_err(|e| format!("创建自启动任务失败: {}", e))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// 移除开机自启动任务
#[tauri::command]
pub async fn remove_admin_autostart() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        crate::admin_helper::remove_admin_task("VerdentAccountManager")
            .map_err(|e| format!("移除自启动任务失败: {}", e))
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

// ============================================================================
// Verdent.exe 路径管理命令
// ============================================================================

/// 选择并保存 Verdent.exe 路径
/// 注意：需要在前端调用文件选择对话框，然后将路径传递给这个命令
#[tauri::command]
pub async fn select_verdent_exe_path(path: Option<String>) -> Result<Option<String>, String> {
    println!("[*] 处理 Verdent.exe 路径选择...");
    
    match path {
        Some(file_path) => {
            println!("[✓] 用户选择了: {}", file_path);
            
            // 验证文件是否存在
            let path_buf = std::path::PathBuf::from(&file_path);
            if !path_buf.exists() {
                return Err("选择的文件不存在".to_string());
            }
            
            // 验证文件名是否为 Verdent.exe
            if !path_buf.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.eq_ignore_ascii_case("Verdent.exe"))
                .unwrap_or(false) {
                return Err("请选择 Verdent.exe 文件".to_string());
            }
            
            // 保存到设置
            match SettingsManager::new() {
                Ok(settings_mgr) => {
                    if let Err(e) = settings_mgr.update_verdent_exe_path(Some(file_path.clone())) {
                        return Err(format!("保存路径失败: {}", e));
                    }
                    println!("[✓] 路径已保存到设置");
                },
                Err(e) => {
                    return Err(format!("初始化设置管理器失败: {}", e));
                }
            }
            
            Ok(Some(file_path))
        },
        None => {
            println!("[!] 用户未选择文件");
            Ok(None)
        }
    }
}

/// 获取当前保存的 Verdent.exe 路径
#[tauri::command]
pub async fn get_verdent_exe_path() -> Result<Option<String>, String> {
    match SettingsManager::new() {
        Ok(settings_mgr) => {
            match settings_mgr.load() {
                Ok(settings) => Ok(settings.verdent_exe_path),
                Err(e) => Err(format!("加载设置失败: {}", e))
            }
        },
        Err(e) => Err(format!("初始化设置管理器失败: {}", e))
    }
}

/// 清除保存的 Verdent.exe 路径
#[tauri::command]
pub async fn clear_verdent_exe_path() -> Result<(), String> {
    match SettingsManager::new() {
        Ok(settings_mgr) => {
            if let Err(e) = settings_mgr.update_verdent_exe_path(None) {
                return Err(format!("清除路径失败: {}", e));
            }
            Ok(())
        },
        Err(e) => Err(format!("初始化设置管理器失败: {}", e))
    }
}

/// 调试命令：打印完整的配置文件内容
#[tauri::command]
pub async fn debug_print_settings() -> Result<String, String> {
    println!("\n[DEBUG] ========== 打印完整配置文件 ==========");

    match SettingsManager::new() {
        Ok(settings_mgr) => {
            match settings_mgr.load() {
                Ok(settings) => {
                    let json = serde_json::to_string_pretty(&settings)
                        .map_err(|e| format!("序列化失败: {}", e))?;

                    println!("{}", json);
                    println!("==============================================\n");

                    Ok(json)
                },
                Err(e) => Err(format!("加载配置失败: {}", e))
            }
        },
        Err(e) => Err(format!("初始化设置管理器失败: {}", e))
    }
}

/// 检查 Verdent.exe 是否可用
#[tauri::command]
pub async fn check_verdent_exe_available() -> Result<bool, String> {
    println!("[*] 检查 Verdent.exe 是否可用...");

    let mut manager = VerdentClientManager::new();

    // 获取用户自定义的路径
    let custom_path = match SettingsManager::new() {
        Ok(settings_mgr) => {
            match settings_mgr.load() {
                Ok(settings) => {
                    println!("[*] 从配置文件读取到的路径: {:?}", settings.verdent_exe_path);
                    settings.verdent_exe_path
                },
                Err(e) => {
                    println!("[!] 加载配置文件失败: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("[!] 初始化设置管理器失败: {}", e);
            None
        }
    };

    println!("[*] 将使用的自定义路径: {:?}", custom_path);

    // 尝试查找 Verdent.exe
    match manager.find_verdent_path(custom_path.as_deref()) {
        Ok(path) => {
            println!("[✓] 找到 Verdent.exe: {:?}", path);
            Ok(true)
        },
        Err(e) => {
            println!("[×] 未找到 Verdent.exe: {}", e);
            Ok(false)
        }
    }
}

// ============================================================================
// 机器码管理命令
// ============================================================================

/// 获取机器码信息
#[tauri::command]
pub async fn get_machine_guid_info() -> Result<crate::machine_guid::MachineGuidInfo, String> {
    use crate::machine_guid::MachineGuidManager;

    let manager = MachineGuidManager::new()?;
    manager.get_info()
}

/// 备份当前机器码
#[tauri::command]
pub async fn backup_machine_guid() -> Result<String, String> {
    use crate::machine_guid::MachineGuidManager;

    let manager = MachineGuidManager::new()?;
    manager.backup_current_guid()
}

/// 重置机器码为新的随机 GUID
#[tauri::command]
pub async fn reset_machine_guid() -> Result<String, String> {
    use crate::machine_guid::MachineGuidManager;

    let manager = MachineGuidManager::new()?;
    manager.reset_to_new_guid()
}

/// 恢复备份的机器码
#[tauri::command]
pub async fn restore_machine_guid() -> Result<String, String> {
    use crate::machine_guid::MachineGuidManager;

    let manager = MachineGuidManager::new()?;
    manager.restore_backup_guid()
}

/// 删除机器码备份
#[tauri::command]
pub async fn delete_machine_guid_backup() -> Result<(), String> {
    use crate::machine_guid::MachineGuidManager;

    let manager = MachineGuidManager::new()?;
    manager.delete_backup()
}

/// 获取所有支持的编辑器信息
#[tauri::command]
pub async fn get_all_editors_info() -> Result<Vec<crate::editor_config::EditorInfo>, String> {
    use crate::editor_config::get_all_editors_info;
    
    Ok(get_all_editors_info())
}

/// 获取已安装的编辑器列表
#[tauri::command]
pub async fn get_installed_editors() -> Result<Vec<String>, String> {
    use crate::editor_config::get_installed_editors;
    
    Ok(get_installed_editors()
        .iter()
        .map(|e| e.display_name().to_string())
        .collect())
}

#[tauri::command]
pub async fn check_for_updates(current_version: String) -> Result<crate::updater::UpdateInfo, String> {
    crate::updater::check_for_updates(&current_version).await
}

#[tauri::command]
pub async fn skip_version(version: String) -> Result<(), String> {
    crate::updater::save_skipped_version(version)
}

#[tauri::command]
pub async fn clear_skipped_version() -> Result<(), String> {
    crate::updater::clear_skipped_version()
}
