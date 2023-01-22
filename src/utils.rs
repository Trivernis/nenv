use std::{
    path::{Path, PathBuf},
    time::Duration,
};

use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

pub fn progress_bar(total: u64) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg} {spinner}\n[{wide_bar}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})",
            )
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
    pb
}

#[cfg(not(target_os = "windows"))]
pub fn progress_spinner() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg} {spinner}")
            .unwrap(),
    );
    pb.enable_steady_tick(Duration::from_millis(50));
    pb
}

pub fn prompt<S: ToString>(default: bool, prompt: S) -> bool {
    Confirm::new()
        .with_prompt(prompt.to_string())
        .default(default)
        .interact()
        .unwrap()
}

pub fn find_in_parents<P: AsRef<Path>>(origin: PathBuf, name: P) -> Option<PathBuf> {
    for part in dir_parts(origin) {
        let file = part.join(&name);
        if file.exists() {
            return Some(file);
        }
    }

    None
}

/// Returns a list of paths for the current dir up to the very top
pub fn dir_parts(path: PathBuf) -> Vec<PathBuf> {
    let mut current: &Path = &path;
    let mut parts = vec![path.to_owned()];

    while let Some(parent) = current.parent() {
        current = parent;
        parts.push(parent.to_owned())
    }

    parts
}
