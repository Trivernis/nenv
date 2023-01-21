mod consts;
mod download;
pub mod error;
mod utils;
mod web_api;

pub enum Version {
    Latest,
    Lts,
    Specific(u8, Option<u8>, Option<u16>),
}

pub fn install(version: Version) {}
