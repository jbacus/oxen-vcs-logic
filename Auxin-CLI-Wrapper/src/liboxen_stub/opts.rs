use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AddOpts {
    pub paths: Vec<PathBuf>,
    pub is_remote: bool,
    pub directory: Option<PathBuf>,
}
