use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct NodePath {
    base: PathBuf,
}

impl NodePath {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    #[cfg(not(windows))]
    pub fn bin(&self) -> PathBuf {
        self.base.join("bin")
    }

    #[cfg(windows)]
    pub fn bin(&self) -> PathBuf {
        self.base.to_owned()
    }

    #[cfg(not(windows))]
    pub fn lib(&self) -> PathBuf {
        self.base.join("lib")
    }

    #[cfg(windows)]
    pub fn lib(&self) -> PathBuf {
        self.base.to_owned()
    }

    pub fn node_modules(&self) -> PathBuf {
        self.lib().join("node_modules")
    }
}
