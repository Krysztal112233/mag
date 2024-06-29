use serde::{Deserialize, Serialize};
use std::io;
use temp_file::TempFile;
use thiserror::Error;

pub mod modrinth;
pub mod ratelimit;

#[derive(Debug, Error)]
pub enum FetcherError {
    #[error("{0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    IO(#[from] io::Error),

    #[error("{0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("reached ratelimit")]
    RatelimitReached,

    #[error("No ratelimit")]
    NoRatelimit,

    #[error("parse failed")]
    ParseFailed,

    #[error("no body")]
    NoBody,
}

type Result<T> = std::result::Result<T, FetcherError>;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModFileInfomation {
    pub filename:String,
    pub id: String,
    pub url: String,
    pub sha256: String,
}

pub trait Fetcher {
    async fn fetch<T>(&self, version_id: T) -> Result<ModFileInfomation>
    where
        T: Into<String>;

    async fn download<T>(&self, version_id: T) -> Result<TempFile>
    where
        T: Into<String>;
}
