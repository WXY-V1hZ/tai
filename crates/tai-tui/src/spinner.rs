use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    /// 创建一个新的加载 Spinner，显示指定消息
    pub fn new(message: &str) -> Self {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ")
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(Duration::from_millis(100));
        Self { pb }
    }

    /// 更新 Spinner 显示的消息
    pub fn set_message(&self, message: &str) {
        self.pb.set_message(message.to_string());
    }

    /// 完成并显示成功消息（绿色勾号）
    pub fn finish_with_message(&self, message: &str) {
        self.pb.finish_with_message(format!("✓ {}", message));
    }

    /// 完成并清除 Spinner
    pub fn finish_and_clear(&self) {
        self.pb.finish_and_clear();
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.pb.is_finished() {
            self.pb.finish_and_clear();
        }
    }
}
