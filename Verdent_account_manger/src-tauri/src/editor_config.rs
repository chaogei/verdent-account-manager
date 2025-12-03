use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EditorType {
    VSCode,
    Windsurf,
    Cursor,
    Trae,
    Qoder,
    Kiro,
}

impl EditorType {
    /// 获取编辑器显示名称
    pub fn display_name(&self) -> &str {
        match self {
            EditorType::VSCode => "VS Code",
            EditorType::Windsurf => "Windsurf",
            EditorType::Cursor => "Cursor",
            EditorType::Trae => "Trae",
            EditorType::Qoder => "Qoder",
            EditorType::Kiro => "Kiro",
        }
    }

    /// 获取编辑器文件夹名称
    pub fn folder_name(&self) -> &str {
        match self {
            EditorType::VSCode => "Code",
            EditorType::Windsurf => "Windsurf",
            EditorType::Cursor => "Cursor",
            EditorType::Trae => "Trae",
            EditorType::Qoder => "Qoder",
            EditorType::Kiro => "Kiro",
        }
    }

    /// 获取编辑器的 globalStorage 路径
    pub fn get_global_storage_path(&self) -> Option<PathBuf> {
        let folder_name = self.folder_name();
        
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|config| {
                config.join(folder_name).join("User").join("globalStorage")
            })
        }
        
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir().map(|home| {
                home.join("Library")
                    .join("Application Support")
                    .join(folder_name)
                    .join("User")
                    .join("globalStorage")
            })
        }
        
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir().map(|config| {
                config.join(folder_name).join("User").join("globalStorage")
            })
        }
    }

    /// 获取编辑器的 storage.json 路径
    pub fn get_storage_json_path(&self) -> Option<PathBuf> {
        self.get_global_storage_path().map(|path| path.join("storage.json"))
    }

    /// 获取编辑器的 state.vscdb 路径
    pub fn get_state_db_path(&self) -> Option<PathBuf> {
        self.get_global_storage_path().map(|path| path.join("state.vscdb"))
    }

    /// 获取编辑器的 verdentai.verdent 存储路径
    pub fn get_verdent_storage_path(&self) -> Option<PathBuf> {
        self.get_global_storage_path().map(|path| path.join("verdentai.verdent"))
    }

    /// 获取所有支持的编辑器列表
    pub fn all() -> Vec<EditorType> {
        vec![
            EditorType::VSCode,
            EditorType::Windsurf,
            EditorType::Cursor,
            EditorType::Trae,
            EditorType::Qoder,
            EditorType::Kiro,
        ]
    }

    /// 从字符串解析编辑器类型
    pub fn from_string(s: &str) -> Option<EditorType> {
        match s.to_lowercase().as_str() {
            "vscode" | "code" => Some(EditorType::VSCode),
            "windsurf" => Some(EditorType::Windsurf),
            "cursor" => Some(EditorType::Cursor),
            "trae" => Some(EditorType::Trae),
            "qoder" => Some(EditorType::Qoder),
            "kiro" => Some(EditorType::Kiro),
            _ => None,
        }
    }
}

/// 编辑器配置信息
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorInfo {
    pub editor_type: EditorType,
    pub display_name: String,
    pub is_installed: bool,
    pub storage_path: Option<String>,
    pub state_db_path: Option<String>,
}

impl EditorInfo {
    pub fn new(editor_type: EditorType) -> Self {
        let storage_path = editor_type.get_global_storage_path();
        let state_db_path = editor_type.get_state_db_path();
        
        // 检查编辑器是否安装（通过检查存储路径是否存在）
        let is_installed = storage_path.as_ref().map_or(false, |p| p.exists());
        
        EditorInfo {
            display_name: editor_type.display_name().to_string(),
            editor_type,
            is_installed,
            storage_path: storage_path.map(|p| p.to_string_lossy().to_string()),
            state_db_path: state_db_path.map(|p| p.to_string_lossy().to_string()),
        }
    }
}

/// 获取所有编辑器的信息
pub fn get_all_editors_info() -> Vec<EditorInfo> {
    EditorType::all()
        .into_iter()
        .map(EditorInfo::new)
        .collect()
}

/// 获取已安装的编辑器列表
pub fn get_installed_editors() -> Vec<EditorType> {
    EditorType::all()
        .into_iter()
        .filter(|editor| {
            editor.get_global_storage_path()
                .map_or(false, |p| p.exists())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_editor_paths() {
        for editor in EditorType::all() {
            println!("{} paths:", editor.display_name());
            if let Some(path) = editor.get_global_storage_path() {
                println!("  GlobalStorage: {}", path.display());
            }
            if let Some(path) = editor.get_storage_json_path() {
                println!("  storage.json: {}", path.display());
            }
            if let Some(path) = editor.get_state_db_path() {
                println!("  state.vscdb: {}", path.display());
            }
        }
    }

    #[test]
    fn test_editor_from_string() {
        assert_eq!(EditorType::from_string("vscode"), Some(EditorType::VSCode));
        assert_eq!(EditorType::from_string("Code"), Some(EditorType::VSCode));
        assert_eq!(EditorType::from_string("windsurf"), Some(EditorType::Windsurf));
        assert_eq!(EditorType::from_string("unknown"), None);
    }
}
