use std::fs;
use std::path::PathBuf;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// 获取日志目录路径
fn log_dir() -> PathBuf {
    let tai_dir = dirs_next::home_dir()
        .expect("无法获取用户主目录")
        .join(".tai")
        .join("logs");
    
    if !tai_dir.exists() {
        fs::create_dir_all(&tai_dir).expect("无法创建日志目录");
    }
    
    tai_dir
}

/// 清理旧的日志文件，只保留最新的 max_files 个
fn cleanup_old_logs(log_dir: &PathBuf, max_files: usize) {
    if let Ok(entries) = fs::read_dir(log_dir) {
        let mut log_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "log")
                    .unwrap_or(false)
            })
            .collect();

        // 按修改时间排序（最新的在前）
        log_files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        // 删除超过 max_files 的旧文件
        for file in log_files.iter().skip(max_files) {
            if let Err(e) = fs::remove_file(file.path()) {
                eprintln!("清理日志文件失败 {:?}: {}", file.path(), e);
            }
        }
    }
}

/// 初始化日志系统
/// 
/// - 日志按小时滚动
/// - 最多保留 10 个日志文件
/// - 同时输出到控制台和文件
pub fn init_logging() {
    let log_path = log_dir();
    
    // 清理旧日志
    cleanup_old_logs(&log_path, 10);

    // 创建文件日志 appender（按小时滚动）
    let file_appender = RollingFileAppender::builder()
        .rotation(Rotation::HOURLY)
        .filename_prefix("tai")
        .filename_suffix("log")
        .build(&log_path)
        .expect("无法创建日志文件");

    // 文件日志层（记录所有 DEBUG 及以上级别）
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_file(true);

    // 控制台日志层（只记录 INFO 及以上级别，格式简洁）
    let console_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_line_number(false)
        .with_file(false)
        .compact();

    // 环境变量过滤器，默认控制台 INFO，文件 DEBUG
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("tai=debug,warn"));

    // 组合订阅器
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer.with_filter(tracing_subscriber::filter::LevelFilter::INFO))
        .with(file_layer.with_filter(tracing_subscriber::filter::LevelFilter::DEBUG))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_dir_creation() {
        let dir = log_dir();
        assert!(dir.exists());
        assert!(dir.ends_with(".tai/logs"));
    }
}
