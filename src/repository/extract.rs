use std::{
    fs::{self, File},
    io::{self, BufReader},
    path::Path,
};

use libflate::gzip::Decoder;
use miette::Diagnostic;
use tar::Archive;
use thiserror::Error;
use zip::ZipArchive;

use crate::utils::{progress_bar, progress_spinner};
type ExtractResult<T> = Result<T, ExtractError>;

#[derive(Error, Debug, Diagnostic)]
pub enum ExtractError {
    #[error("IO error when extracting: {0}")]
    Io(
        #[from]
        #[source]
        io::Error,
    ),
    #[error("Failed to extract zip: {0}")]
    Zip(
        #[from]
        #[source]
        zip::result::ZipError,
    ),
}

pub fn extract_file(src: &Path, dst: &Path) -> ExtractResult<()> {
    #[cfg(target_os = "windows")]
    extract_zip(src, dst)?;
    #[cfg(not(target_os = "windows"))]
    extract_tar_gz(src, dst)?;

    Ok(())
}

fn extract_tar_gz(src: &Path, dst: &Path) -> ExtractResult<()> {
    let mut reader = BufReader::new(File::open(src)?);
    let mut decoder = Decoder::new(reader)?;
    let mut archive = Archive::new(decoder);
    let pb = progress_spinner();
    pb.set_message("Extracting tar.gz archive");

    archive.unpack(dst)?;
    pb.finish_with_message("Archive extracted.");

    Ok(())
}

fn extract_zip(src: &Path, dst: &Path) -> ExtractResult<()> {
    let mut archive = ZipArchive::new(File::open(src)?)?;

    let pb = progress_bar(archive.len() as u64);
    pb.set_message("Extracting zip archive");

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let Some(path) = file.enclosed_name() else {
                    tracing::error!(
                        "Cannot extract {:?} because it has an invalid name",
                        file.name()
                    );
            continue;
        };
        let output_path = dst.join(path);
        if (*file.name()).ends_with('/') {
            tracing::debug!("Creating directory {output_path:?}");
            fs::create_dir_all(output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                if !parent.exists() {
                    tracing::debug!("Creating parent directory {parent:?}");
                    fs::create_dir_all(parent)?;
                }
            }
            let mut file_output = File::create(&output_path)?;
            tracing::debug!("Extracting to {output_path:?}");
            io::copy(&mut file, &mut file_output)?;
        }
        pb.tick()
    }
    pb.finish_with_message("Archive extracted.");

    Ok(())
}
