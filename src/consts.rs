use lazy_static::lazy_static;
use std::path::PathBuf;

pub const NODE_DIST_URL: &str = "https://nodejs.org/dist";
#[cfg(not(windows))]
pub const SEARCH_PATH_SEPARATOR: &str = ":";
#[cfg(windows)]
pub const SEARCH_PATH_SEPARATOR: &str = ";";

lazy_static! {
    pub static ref CFG_DIR: PathBuf = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(".config"))
        .join(PathBuf::from("nenv"));
    pub static ref DATA_DIR: PathBuf = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".data"))
        .join(PathBuf::from("nenv"));
    pub static ref CACHE_DIR: PathBuf = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join(PathBuf::from("nenv"));
    pub static ref CFG_FILE_PATH: PathBuf = CFG_DIR.join("config.toml");
    pub static ref VERSION_FILE_PATH: PathBuf = CACHE_DIR.join("versions.cache");
    pub static ref INSTALLED_VERSION_FILE: PathBuf = DATA_DIR.join("installed_versions");
    pub static ref BIN_DIR: PathBuf = DATA_DIR.join("bin");
    pub static ref NODE_VERSIONS_DIR: PathBuf = DATA_DIR.join("versions");
    pub static ref NODE_ARCHIVE_SUFFIX: String = format!("-{OS}-{ARCH}.{ARCHIVE_TYPE}");
}

macro_rules! map_arch {
    ($($arch:literal => $node_arch: literal),+) => {
        map_arch!($($arch => $node_arch,)+);
    };
    ($($arch:literal => $node_arch: literal),+,) => {
        $(
            #[cfg(target_arch = $arch)]
            pub const ARCH: &'static str = $node_arch;
        )+
    };
}

map_arch!(
    "x86_64" => "x64",
    "x86" => "x86",
    "arm" => "armv7l",
    "aarch64" => "arm64",
    "riscv32" => "armv7l",
    "powerpc64" => "ppc64",
    "powerpc64le" => "ppc64le",
    "s390x" => "s390x",
);

macro_rules! map_os {
    ($($os:literal => $node_os: literal),+) => {
        map_arch!($($os => $node_os,)+);
    };
    ($($os:literal => $node_os: literal),+,) => {
        $(
            #[cfg(target_os = $os)]
            pub const OS: &'static str = $node_os;
        )+
    };
}

map_os!(
    "linux" => "linux",
    "windows" => "win",
    "macos" => "darwin",
);

#[cfg(not(target_os = "windows"))]
pub const ARCHIVE_TYPE: &str = "tar.gz";

#[cfg(target_os = "windows")]
pub const ARCHIVE_TYPE: &'static str = "zip";
