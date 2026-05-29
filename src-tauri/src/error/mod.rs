#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("STT error: {0}")]
    Stt(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Permission denied: {0}")]
    Permission(String),
    #[error("Clipboard error: {0}")]
    Clipboard(String),
    #[error("Hotkey error: {0}")]
    Hotkey(String),
}

impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = s.serialize_map(Some(2))?;
        map.serialize_entry(
            "code",
            match self {
                AppError::Provider(_) => "Provider",
                AppError::Stt(_) => "Stt",
                AppError::Storage(_) => "Storage",
                AppError::Permission(_) => "Permission",
                AppError::Clipboard(_) => "Clipboard",
                AppError::Hotkey(_) => "Hotkey",
            },
        )?;
        map.serialize_entry("message", &self.to_string())?;
        map.end()
    }
}
