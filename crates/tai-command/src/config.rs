use tai_core::{TaiConfig, TaiError, TaiResult};
use tai_tui::{show_settings, SettingItem, SettingValue};
use tracing::debug;

pub struct ConfigCommand;

impl ConfigCommand {
    pub async fn handle(self) -> TaiResult<()> {
        let config = TaiConfig::load()?;
        let items = config_to_items(&config);

        match show_settings(items) {
            Ok(Some(updated)) => {
                let new_config = items_to_config(&config, &updated);
                new_config.save()?;
                debug!("配置已保存");
                println!("  ✓ 配置已保存至 ~/.tai/config.json");
            }
            Ok(None) => {
                debug!("用户取消了配置修改");
            }
            Err(e) => {
                return Err(TaiError::Other(format!("TUI 错误: {}", e)));
            }
        }

        Ok(())
    }
}

const THEMES: &[&str] = &["默认", "暗色", "亮色"];

fn config_to_items(config: &TaiConfig) -> Vec<SettingItem> {
    let theme_idx = THEMES
        .iter()
        .position(|&t| t == config.output_theme)
        .unwrap_or(0);

    vec![
        SettingItem::bool("show_markdown_view", "回答后展示 Markdown 渲染", config.show_markdown_view),
        SettingItem::bool("auto_copy_command",  "自动复制命令到剪贴板",    config.auto_copy_command),
        SettingItem::bool("save_history",        "自动保存历史记录",         config.save_history),
        SettingItem::bool("show_reasoning",      "显示 AI 思考过程",         config.show_reasoning),
        SettingItem::bool("compact_output",      "精简输出模式",             config.compact_output),
        SettingItem::bool("debug_logging",       "启用调试日志",             config.debug_logging),
        SettingItem::int(
            "max_history_count",
            "历史记录最多保留条数",
            config.max_history_count as i64,
            1,
            500,
        ),
        SettingItem::select(
            "output_theme",
            "输出主题",
            THEMES.iter().map(|s| s.to_string()).collect(),
            theme_idx,
        ),
    ]
}

fn items_to_config(base: &TaiConfig, items: &[SettingItem]) -> TaiConfig {
    let mut config = base.clone();
    for item in items {
        match (item.key.as_str(), &item.value) {
            ("show_markdown_view", SettingValue::Bool(v))  => config.show_markdown_view = *v,
            ("auto_copy_command",  SettingValue::Bool(v))  => config.auto_copy_command  = *v,
            ("save_history",       SettingValue::Bool(v))  => config.save_history       = *v,
            ("show_reasoning",     SettingValue::Bool(v))  => config.show_reasoning     = *v,
            ("compact_output",     SettingValue::Bool(v))  => config.compact_output     = *v,
            ("debug_logging",      SettingValue::Bool(v))  => config.debug_logging      = *v,
            ("max_history_count",  SettingValue::Int { value, .. }) => {
                config.max_history_count = (*value).max(1) as u32;
            }
            ("output_theme", SettingValue::Select { selected, options }) => {
                config.output_theme = options[*selected].clone();
            }
            _ => {}
        }
    }
    config
}
