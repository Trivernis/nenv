use std::path::PathBuf;

pub struct NodePath {
    base: PathBuf,
}

impl NodePath {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    pub fn bin(&self) -> PathBuf {
        self.base.join("bin")
    }
}
