use super::*;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Image {
    pub path: PathBuf, // static path
}
