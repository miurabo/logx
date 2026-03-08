/// logxのアプリケーションエラー型
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum LogxError {
    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("Permission denied: {path} (try: sudo logx ...)")]
    PermissionDenied { path: String },

    #[error("File is empty: {path}")]
    EmptyFile { path: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),
}
