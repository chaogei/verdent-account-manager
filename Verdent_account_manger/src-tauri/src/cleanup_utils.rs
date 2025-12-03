use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use rusqlite::Connection;
use serde_json::{Value, Map};
use uuid::Uuid;
use dirs;
use crate::editor_config::EditorType;

/// 清空指定文件夹的内容（保留文件夹本身）
pub fn clear_directory_contents(path: &Path) -> io::Result<usize> {
    if !path.exists() {
        println!("[!] 文件夹不存在，跳过: {}", path.display());
        return Ok(0);
    }

    if !path.is_dir() {
        println!("[!] 路径不是文件夹，跳过: {}", path.display());
        return Ok(0);
    }

    let mut count = 0;
    let entries = fs::read_dir(path)?;
    
    for entry in entries {
        let entry = entry?;
        let entry_path = entry.path();
        
        if entry_path.is_dir() {
            // 递归删除子文件夹
            match fs::remove_dir_all(&entry_path) {
                Ok(_) => {
                    count += 1;
                    println!("    [✓] 删除文件夹: {}", entry_path.display());
                }
                Err(e) => {
                    eprintln!("    [×] 无法删除文件夹 {}: {}", entry_path.display(), e);
                }
            }
        } else {
            // 删除文件
            match fs::remove_file(&entry_path) {
                Ok(_) => {
                    count += 1;
                    println!("    [✓] 删除文件: {}", entry_path.display());
                }
                Err(e) => {
                    eprintln!("    [×] 无法删除文件 {}: {}", entry_path.display(), e);
                }
            }
        }
    }
    
    Ok(count)
}

/// 获取需要清理的文件夹路径列表（旧版本，仅 VS Code）
pub fn get_cleanup_directories() -> Vec<PathBuf> {
    get_cleanup_directories_for_editors(&[EditorType::VSCode])
}

/// 获取指定编辑器的清理文件夹路径列表
pub fn get_cleanup_directories_for_editors(editors: &[EditorType]) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Some(home) = dirs::home_dir() {
        // .verdent 目录下的文件夹（所有编辑器共享）
        paths.push(home.join(".verdent").join("projects"));
        paths.push(home.join(".verdent").join("logs"));
        paths.push(home.join(".verdent").join("checkpoints").join("checkpoints"));
    }
    
    // 为每个选中的编辑器添加清理路径
    for editor in editors {
        if let Some(verdent_storage) = editor.get_verdent_storage_path() {
            paths.push(verdent_storage.join("tasks"));
            paths.push(verdent_storage.join("checkpoints"));
        }
    }
    
    paths
}

/// 清空所有指定的文件夹内容（旧版本，仅 VS Code）
pub fn clear_all_verdent_directories() -> (usize, Vec<String>) {
    clear_verdent_directories_for_editors(&[EditorType::VSCode])
}

/// 清空指定编辑器的文件夹内容
pub fn clear_verdent_directories_for_editors(editors: &[EditorType]) -> (usize, Vec<String>) {
    let directories = get_cleanup_directories_for_editors(editors);
    let mut total_deleted = 0;
    let mut cleaned_paths = Vec::new();
    
    // 记录清理的编辑器
    if !editors.is_empty() {
        let editor_names: Vec<String> = editors.iter()
            .map(|e| e.display_name().to_string())
            .collect();
        println!("[*] 清理以下编辑器的存储: {}", editor_names.join(", "));
    }
    
    for dir in directories {
        println!("[*] 清理文件夹: {}", dir.display());
        match clear_directory_contents(&dir) {
            Ok(count) => {
                total_deleted += count;
                cleaned_paths.push(format!("清理文件夹: {} ({}个项目)", dir.display(), count));
                println!("    [✓] 已清理 {} 个项目", count);
            }
            Err(e) => {
                eprintln!("    [!] 清理失败: {}", e);
            }
        }
    }
    
    (total_deleted, cleaned_paths)
}

/// 删除用户目录下的 .verdent 文件夹
pub fn delete_verdent_home_directory() -> (usize, Vec<String>) {
    let mut deleted_count = 0;
    let mut deleted_paths = Vec::new();
    
    if let Some(home) = dirs::home_dir() {
        let verdent_dir = home.join(".verdent");
        println!("[*] 删除用户目录下的 .verdent 文件夹: {}", verdent_dir.display());
        
        if verdent_dir.exists() {
            match fs::remove_dir_all(&verdent_dir) {
                Ok(_) => {
                    deleted_count += 1;
                    deleted_paths.push(format!("删除文件夹: {}", verdent_dir.display()));
                    println!("    [✓] 已删除 .verdent 文件夹");
                }
                Err(e) => {
                    eprintln!("    [×] 无法删除 .verdent 文件夹: {}", e);
                }
            }
        } else {
            println!("    [!] .verdent 文件夹不存在，跳过");
        }
    } else {
        eprintln!("[!] 无法获取用户主目录");
    }
    
    (deleted_count, deleted_paths)
}

/// 获取 VS Code state.vscdb 数据库路径
pub fn get_vscode_state_db_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config| {
        config
            .join("Code")
            .join("User")
            .join("globalStorage")
            .join("state.vscdb")
    })
}

/// 重置 VS Code state.vscdb 中的 deviceId（旧版本，仅 VS Code）
pub fn reset_vscode_device_id(generate_new: bool) -> Result<String, String> {
    reset_editor_device_id(&EditorType::VSCode, generate_new)
        .map(|(_, device_id)| device_id)
}

/// 重置单个编辑器的 state.vscdb 中的 deviceId
pub fn reset_editor_device_id(editor: &EditorType, generate_new: bool) -> Result<(String, String), String> {
    let db_path = editor.get_state_db_path()
        .ok_or_else(|| format!("无法获取 {} 数据库路径", editor.display_name()))?;
    
    if !db_path.exists() {
        return Err(format!("{} 数据库文件不存在: {}", editor.display_name(), db_path.display()));
    }
    
    println!("[*] 打开 {} 数据库: {}", editor.display_name(), db_path.display());
    
    // 打开数据库连接
    let conn = Connection::open(&db_path)
        .map_err(|e| format!("打开数据库失败: {}", e))?;
    
    // 查询 VerdentAI.verdent 的记录
    let query = "SELECT value FROM ItemTable WHERE key = 'VerdentAI.verdent'";
    let mut stmt = conn.prepare(query)
        .map_err(|e| format!("准备查询失败: {}", e))?;
    
    let mut rows = stmt.query([])
        .map_err(|e| format!("查询失败: {}", e))?;
    
    if let Some(row) = rows.next().map_err(|e| format!("读取行失败: {}", e))? {
        let value_str: String = row.get(0)
            .map_err(|e| format!("获取值失败: {}", e))?;
        
        // 解析 JSON
        let mut json_value: Value = serde_json::from_str(&value_str)
            .map_err(|e| format!("解析 JSON 失败: {}", e))?;
        
        // 生成新的 deviceId
        let new_device_id = if generate_new {
            Uuid::new_v4().to_string()
        } else {
            "00000000-0000-0000-0000-000000000000".to_string()
        };
        
        // 修改 deviceId
        if let Value::Object(ref mut map) = json_value {
            // 查找并修改 verdent-ai-agent-config.deviceId
            if let Some(Value::Object(config)) = map.get_mut("verdent-ai-agent-config") {
                config.insert("deviceId".to_string(), Value::String(new_device_id.clone()));
                println!("[*] 找到并更新 deviceId: {}", new_device_id);
            } else {
                // 如果没有找到，创建这个字段
                let mut config = Map::new();
                config.insert("deviceId".to_string(), Value::String(new_device_id.clone()));
                map.insert("verdent-ai-agent-config".to_string(), Value::Object(config));
                println!("[*] 创建并设置 deviceId: {}", new_device_id);
            }
        }
        
        // 将修改后的 JSON 转回字符串
        let updated_value = serde_json::to_string(&json_value)
            .map_err(|e| format!("序列化 JSON 失败: {}", e))?;
        
        // 更新数据库
        let update_query = "UPDATE ItemTable SET value = ?1 WHERE key = 'VerdentAI.verdent'";
        conn.execute(update_query, [&updated_value])
            .map_err(|e| format!("更新数据库失败: {}", e))?;
        
        println!("[✓] {} deviceId 已重置为: {}", editor.display_name(), new_device_id);
        Ok((editor.display_name().to_string(), new_device_id))
    } else {
        // 如果没有找到记录，创建一个新记录
        let new_device_id = if generate_new {
            Uuid::new_v4().to_string()
        } else {
            "00000000-0000-0000-0000-000000000000".to_string()
        };
        
        let mut new_value = Map::new();
        let mut config = Map::new();
        config.insert("deviceId".to_string(), Value::String(new_device_id.clone()));
        new_value.insert("verdent-ai-agent-config".to_string(), Value::Object(config));
        
        let value_str = serde_json::to_string(&Value::Object(new_value))
            .map_err(|e| format!("序列化新 JSON 失败: {}", e))?;
        
        let insert_query = "INSERT INTO ItemTable (key, value) VALUES ('VerdentAI.verdent', ?1)";
        conn.execute(insert_query, [&value_str])
            .map_err(|e| format!("插入新记录失败: {}", e))?;
        
        println!("[✓] 创建新记录，deviceId: {}", new_device_id);
        Ok((editor.display_name().to_string(), new_device_id))
    }
}

/// 重置多个编辑器的 deviceId
pub fn reset_editors_device_id(editors: &[EditorType], generate_new: bool) -> Vec<(String, Result<String, String>)> {
    let mut results = Vec::new();
    
    for editor in editors {
        let editor_name = editor.display_name().to_string();
        match reset_editor_device_id(editor, generate_new) {
            Ok((_, device_id)) => {
                results.push((editor_name, Ok(device_id)));
            }
            Err(e) => {
                eprintln!("[!] 重置 {} deviceId 失败: {}", editor.display_name(), e);
                results.push((editor_name, Err(e)));
            }
        }
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_clear_directory_contents() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        
        // 创建一些测试文件和子文件夹
        fs::write(temp_path.join("test1.txt"), "test").unwrap();
        fs::write(temp_path.join("test2.txt"), "test").unwrap();
        fs::create_dir(temp_path.join("subdir")).unwrap();
        fs::write(temp_path.join("subdir").join("test3.txt"), "test").unwrap();
        
        // 清理目录内容
        let count = clear_directory_contents(temp_path).unwrap();
        assert_eq!(count, 3); // 2个文件 + 1个文件夹
        
        // 确认目录本身还存在但内容已清空
        assert!(temp_path.exists());
        assert!(temp_path.read_dir().unwrap().count() == 0);
    }
}
