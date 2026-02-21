use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaiError {
    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("未找到 provider 配置，请检查 ~/.tai/providers.json")]
    NoProviderConfig,

    #[error("没有可用的模型，请运行 `tai model` 确认配置")]
    NoActiveModel,

    #[error("未找到模型 `{0}`，运行 `tai model` 查看可用模型")]
    ModelNotFound(String),

    #[error("AI 请求错误: {0}")]
    AiError(String),

    #[error("API Key 无效，请重新输入 {0} 的 API Key")]
    AuthError(String),

    #[error("无法连接到服务器 {0}，请检查 providers.json 中的 base_url 配置")]
    ConnectionError(String),

    #[error("文件操作错误: {0}")]
    FileError(String),

    #[error("网络请求错误: {0}")]
    NetworkError(String),

    #[error("用户输入不能为空")]
    EmptyInput,

    #[error("初始化错误: {0}")]
    InitError(String),

    #[error("{0}")]
    Other(String),
}

impl From<std::io::Error> for TaiError {
    fn from(err: std::io::Error) -> Self {
        tracing::error!("IO 错误: {}", err);
        TaiError::FileError(err.to_string())
    }
}

impl From<serde_json::Error> for TaiError {
    fn from(err: serde_json::Error) -> Self {
        tracing::error!("JSON 解析错误: {}", err);
        TaiError::ConfigError(err.to_string())
    }
}

pub type TaiResult<T> = Result<T, TaiError>;
