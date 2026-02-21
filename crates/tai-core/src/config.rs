use dirs_next::home_dir;
use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tracing::{debug, warn};

use crate::{TaiError, TaiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaiConfig {
    /// ask 命令回答后是否进入可滚动的 Markdown 渲染界面
    pub show_markdown_view: bool,
    /// tai go 命令结果是否自动复制到剪贴板
    pub auto_copy_command: bool,
    /// ask 命令是否自动保存历史记录
    pub save_history: bool,
    /// 是否显示 AI 思考过程（reasoning）
    pub show_reasoning: bool,
    /// 精简输出模式（隐藏辅助信息）
    pub compact_output: bool,
    /// 启用调试日志输出
    pub debug_logging: bool,
    /// 历史记录最多保留条数
    pub max_history_count: u32,
    /// 输出主题
    pub output_theme: String,
}

impl Default for TaiConfig {
    fn default() -> Self {
        Self {
            show_markdown_view: true,
            auto_copy_command: true,
            save_history: true,
            show_reasoning: true,
            compact_output: false,
            debug_logging: false,
            max_history_count: 50,
            output_theme: "默认".to_string(),
        }
    }
}

fn config_path() -> PathBuf {
    home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".tai")
        .join("config.json")
}

impl TaiConfig {
    pub fn load() -> TaiResult<Self> {
        let path = config_path();
        if !path.exists() {
            debug!("配置文件不存在，使用默认配置");
            return Ok(Self::default());
        }
        let content = fs::read_to_string(&path).map_err(|e| {
            TaiError::FileError(format!("无法读取配置文件 {:?}: {}", path, e))
        })?;
        let config: Self = serde_json::from_str(&content).unwrap_or_else(|e| {
            warn!("配置文件解析失败，使用默认配置: {}", e);
            Self::default()
        });
        debug!("配置文件已加载: {:?}", path);
        Ok(config)
    }

    pub fn save(&self) -> TaiResult<()> {
        let path = config_path();
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| {
            TaiError::FileError(format!("无法创建配置目录: {}", e))
        })?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content).map_err(|e| {
            TaiError::FileError(format!("无法写入配置文件 {:?}: {}", path, e))
        })?;
        debug!("配置文件已保存: {:?}", path);
        Ok(())
    }
}
