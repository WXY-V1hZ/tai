use dirs_next::home_dir;
use encoding_rs::GBK;
use std::fmt::Debug;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use tai_core::{TaiError, TaiResult};
use tracing::{error, debug};

#[cfg(target_os = "windows")]
const OS_NAME: &str = "windows";
#[cfg(target_os = "linux")]
const OS_NAME: &str = "linux";
#[cfg(target_os = "macos")]
const OS_NAME: &str = "macos";

#[derive(Debug)]
pub struct InitCommand;

impl InitCommand {
    pub async fn handle(self) -> TaiResult<()> {
        debug!("开始初始化系统信息收集");
        
        // 选择不同命令
        let (info_cmd, info_args): (&str, Vec<&str>) = match OS_NAME {
            "windows" => ("systeminfo", vec![]),
            "linux" => ("uname", vec!["-a"]),
            "macos" => ("system_profiler", vec!["SPHardwareDataType"]),
            _ => ("uname", vec!["-a"]),
        };

        debug!("执行系统命令: {} {:?}", info_cmd, info_args);

        // 运行命令
        let output = Command::new(info_cmd)
            .args(&info_args)
            .output()
            .map_err(|e| {
                error!("执行系统命令失败: {}", e);
                TaiError::InitError(format!("无法执行 {}: {}", info_cmd, e))
            })?;

        let info = if output.status.success() {
            // 在 Windows 上，systeminfo 输出是 GBK 编码
            #[cfg(target_os = "windows")]
            {
                let (decoded, _, _) = GBK.decode(&output.stdout);
                decoded.into_owned()
            }
            #[cfg(not(target_os = "windows"))]
            {
                String::from_utf8_lossy(&output.stdout).to_string()
            }
        } else {
            #[cfg(target_os = "windows")]
            {
                let (decoded, _, _) = GBK.decode(&output.stderr);
                let err_msg = format!("获取系统信息失败: {}", decoded);
                error!("{}", err_msg);
                return Err(TaiError::InitError(err_msg));
            }
            #[cfg(not(target_os = "windows"))]
            {
                let err_msg = format!(
                    "获取系统信息失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                error!("{}", err_msg);
                return Err(TaiError::InitError(err_msg));
            }
        };

        // 获取家目录
        let home_dir = home_dir().ok_or_else(|| {
            error!("无法获取用户主目录");
            TaiError::InitError("无法获取用户主目录".to_string())
        })?;
        let tai_dir = home_dir.join(".tai");

        // 创建~/.tai目录, 如果不存在
        if !tai_dir.exists() {
            debug!("创建目录: {:?}", tai_dir);
            create_dir_all(&tai_dir).map_err(|e| {
                error!("创建 .tai 目录失败: {}", e);
                TaiError::FileError(format!("无法创建 {:?}: {}", tai_dir, e))
            })?;
        }

        // 写入文件，使用UTF-8编码
        let mut file_path = PathBuf::from(&tai_dir);
        file_path.push("sysinfo.txt");

        debug!("写入系统信息到: {:?}", file_path);

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(info.as_bytes())?;
        writer.flush()?;

        println!("init success");
        debug!("初始化完成，系统信息已保存到 {:?}", file_path);

        Ok(())
    }
}
