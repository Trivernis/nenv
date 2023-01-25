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
        self.base.join("bin").canonicalize().unwrap()
    }

    pub fn lib(&self) -> PathBuf {
        self.base.join("lib").canonicalize().unwrap()
    }

    pub fn node_modules(&self) -> PathBuf {
        self.lib().join("node_modules")
    }

    #[cfg(target_os = "windows")]
    pub fn bin(&self) -> PathBuf {
        self.base.to_owned()
    }
}
