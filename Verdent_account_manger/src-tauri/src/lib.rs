mod account_manager;
mod admin_helper;
mod api;
mod cleanup_utils;
mod commands;
mod editor_config;
mod errors;
mod jwt_utils;
mod machine_guid;
mod models;
mod pkce;
mod proxy_manager;
mod settings_manager;
mod storage;
mod verdent_client;
mod vscode_storage;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|#[allow(unused_variables)] app| {
            // 在生产环境中完全禁用 DevTools
            #[cfg(not(debug_assertions))]
            {
                use tauri::Manager;
                // 禁用开发者工具快捷键
                let window = app.get_webview_window("main").unwrap();
                
                // 生产环境：禁用所有开发者工具访问
                window.eval("
                    // 禁用 F12
                    document.addEventListener('keydown', function(e) {
                        if (e.keyCode === 123) {
                            e.preventDefault();
                            return false;
                        }
                    });
                    
                    // 禁用 Ctrl+Shift+I (Windows/Linux) 和 Cmd+Option+I (Mac)
                    document.addEventListener('keydown', function(e) {
                        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.keyCode === 73) {
                            e.preventDefault();
                            return false;
                        }
                    });
                    
                    // 禁用 Ctrl+Shift+J (Windows/Linux) 和 Cmd+Option+J (Mac)
                    document.addEventListener('keydown', function(e) {
                        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.keyCode === 74) {
                            e.preventDefault();
                            return false;
                        }
                    });
                    
                    // 禁用 Ctrl+Shift+C (Windows/Linux) 和 Cmd+Option+C (Mac)
                    document.addEventListener('keydown', function(e) {
                        if ((e.ctrlKey || e.metaKey) && e.shiftKey && e.keyCode === 67) {
                            e.preventDefault();
                            return false;
                        }
                    });
                ").unwrap();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            login_with_token,
            reset_device_identity,
            reset_all_storage,
            get_storage_info,
            open_vscode_callback,
            get_vscode_ids,
            reset_vscode_ids,
            auto_register_accounts,
            get_all_accounts,
            get_app_data_path,
            get_account,
            update_account,
            delete_account,
            update_account_token,
            update_account_quota,
            login_to_vscode,
            login_to_windsurf,
            login_to_cursor,
            login_to_trae,
            login_to_qoder,
            login_to_kiro,
            login_to_verdent_client,
            refresh_account_info,
            open_storage_folder,
            test_login_and_update,
            import_account_by_token,
            import_account_by_credentials,
            get_user_settings,
            save_user_settings,
            update_current_account,
            update_email_privacy,
            update_selected_accounts,
            update_filter_type,
            update_register_config,
            get_proxy_settings,
            save_proxy_settings,
            get_trial_checkout_url,
            open_incognito_browser,
            get_machine_guid_info,
            backup_machine_guid,
            reset_machine_guid,
            restore_machine_guid,
            delete_machine_guid_backup,
            select_verdent_exe_path,
            get_verdent_exe_path,
            clear_verdent_exe_path,
            check_verdent_exe_available,
            check_admin_privileges,
            request_admin_privileges,
            setup_admin_autostart,
            remove_admin_autostart,
            debug_print_settings,
            get_all_editors_info,
            get_installed_editors,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
