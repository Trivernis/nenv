use std::time::Duration;

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
