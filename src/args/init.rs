use dirs_next::home_dir;
use encoding_rs::GBK;
use std::fmt::Debug;
use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;
use std::process::Command;
use std::error::Error;

#[cfg(target_os = "windows")]
const OS_NAME: &str = "windows";
#[cfg(target_os = "linux")]
const OS_NAME: &str = "linux";
#[cfg(target_os = "macos")]
const OS_NAME: &str = "macos";

#[derive(Debug)]
pub struct InitCommand;

impl InitCommand {
    pub async fn handle(self) -> Result<(), Box<dyn Error>> {
        // 选择不同命令
        let (info_cmd, info_args): (&str, Vec<&str>) = match OS_NAME {
            "windows" => ("systeminfo", vec![]),
            "linux" => ("uname", vec!["-a"]),
            "macos" => ("system_profiler", vec!["SPHardwareDataType"]),
            _ => ("uname", vec!["-a"]),
        };

        // 运行命令
        let output = Command::new(info_cmd).args(&info_args).output().unwrap();

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
                format!("获取系统信息失败: {}", decoded)
            }
            #[cfg(not(target_os = "windows"))]
            {
                format!(
                    "获取系统信息失败: {}",
                    String::from_utf8_lossy(&output.stderr)
                )
            }
        };

        // 获取家目录
        let home_dir = home_dir().unwrap();
        let tai_dir = home_dir.join(".tai");

        // 创建~/.tai目录, 如果不存在
        if !tai_dir.exists() {
            create_dir_all(&tai_dir).unwrap();
        }

        // 写入文件，使用UTF-8编码
        let mut file_path = PathBuf::from(&tai_dir);
        file_path.push("sysinfo.txt");

        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(info.as_bytes())?;
        writer.flush()?;

        println!("init success");

        Ok(())
    }
}
