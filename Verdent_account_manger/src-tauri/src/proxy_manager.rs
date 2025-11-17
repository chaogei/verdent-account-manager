use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxySettings {
    pub enabled: bool,
    pub url: String,
}

impl Default for ProxySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::from("http://127.0.0.1:7890"),
        }
    }
}

pub struct ProxyManager;

impl ProxyManager {
    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .expect("无法获取配置目录")
            .join("ai.verdent.account-manager")
            .join("proxy_settings.json")
    }

    pub fn load() -> Result<ProxySettings, String> {
        let config_path = Self::get_config_path();
        
        if !config_path.exists() {
            // 如果文件不存在，返回默认设置
            return Ok(ProxySettings::default());
        }

        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("读取代理设置失败: {}", e))?;
        
        serde_json::from_str(&content)
            .map_err(|e| format!("解析代理设置失败: {}", e))
    }

    pub fn save(settings: &ProxySettings) -> Result<(), String> {
        let config_path = Self::get_config_path();
        
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("创建配置目录失败: {}", e))?;
        }

        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("序列化代理设置失败: {}", e))?;
        
        fs::write(&config_path, content)
            .map_err(|e| format!("保存代理设置失败: {}", e))?;

        Ok(())
    }
}
