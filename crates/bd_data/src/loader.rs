use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    #[error("IO error: {0}")]
    Io(String),
    #[error("RON parse error in {file}: {message}")]
    Ron { file: String, message: String },
}

/// A RON content file with source path for error reporting.
pub struct ContentFile<T> {
    pub path: String,
    pub items: Vec<T>,
}

/// Load a RON file from disk and deserialize into Vec<T>.
pub fn load_ron<T: serde::de::DeserializeOwned>(path: &Path) -> Result<ContentFile<T>, LoadError> {
    let content = std::fs::read_to_string(path).map_err(|e| LoadError::Io(e.to_string()))?;
    let items: Vec<T> = ron::from_str(&content).map_err(|e| LoadError::Ron {
        file: path.to_string_lossy().into_owned(),
        message: e.to_string(),
    })?;
    Ok(ContentFile {
        path: path.to_string_lossy().into_owned(),
        items,
    })
}
