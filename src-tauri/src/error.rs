use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("ADB not found. Please install Android Platform Tools or set ANDROID_HOME environment variable.")]
    AdbNotFound,

    #[error("ADB command failed: {0}")]
    CommandFailed(String),


    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("APK install failed: {0}")]
    InstallFailed(String),
}

// Tauri requires errors to be serializable to send them to the frontend
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("AppError", 2)?;
        s.serialize_field("code", &self.error_code())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

impl AppError {
    fn error_code(&self) -> &'static str {
        match self {
            AppError::AdbNotFound => "ADB_NOT_FOUND",
            AppError::CommandFailed(_) => "COMMAND_FAILED",
            AppError::Io(_) => "IO_ERROR",
            AppError::InstallFailed(_) => "INSTALL_FAILED",
        }
    }
}
