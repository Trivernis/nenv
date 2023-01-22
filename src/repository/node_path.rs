use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NodePath {
    base: PathBuf,
}

impl NodePath {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn bin(&self) -> PathBuf {
        self.base.join("bin")
    }

    #[cfg(target_os = "windows")]
    pub fn bin(&self) -> PathBuf {
        self.base.to_owned()
    }
}
