use super::Fetcher;
use crate::fetcher::{ratelimit::modrinth_ratelimit, FetcherError, ModFileInfomation};
use log::debug;
use reqwest::{header::HeaderMap, Client};
use serde::Deserialize;
use serde_json::Value;
use std::{fs, time::Duration};
use temp_file::TempFile;

#[derive(Debug, Default)]
pub struct ModrinthFetcher(Client);

impl ModrinthFetcher {
    pub fn try_new() -> super::Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert(
            "User-Agent",
            "github.com/krysztal112233/mag (krysztal.huang@outlook.com)"
                .parse()
                .unwrap(),
        );

        let builder = Client::builder()
            .timeout(Duration::from_secs(30))
            .default_headers(headers);

        Ok(ModrinthFetcher(builder.build()?))
    }
}

impl Fetcher for ModrinthFetcher {
    async fn fetch<T>(&self, version_id: T) -> super::Result<super::ModFileInfomation>
    where
        T: Into<String>,
    {
        #[allow(unused)]
        #[derive(Deserialize)]
        struct ModrinthVersionFileDetail {
            hashes: Hashes,
            url: String,
            filename: String,
            primary: bool,
            size: i64,
            file_type: Option<String>,
        }

        #[allow(unused)]
        #[derive(Deserialize)]
        struct Hashes {
            sha512: String,
            sha1: String,
        }

        let version_id: String = version_id.into();

        let version_data_url = format!("https://api.modrinth.com/v2/version/{}", version_id);

        debug!(
            "Construct version url {} for {}.",
            version_data_url, version_id
        );

        let version_json = self.0.get(version_data_url).send().await?.text().await?;
        let version_json = serde_json::from_str::<Value>(&version_json)?;

        let file = {
            // FUCK......
            let file = version_json
                .as_object()
                .unwrap()
                .get("files")
                .unwrap()
                .as_array()
                .unwrap()
                .first()
                .unwrap();

            serde_json::from_value::<ModrinthVersionFileDetail>(file.clone())?
        };

        Ok(ModFileInfomation {
            filename: file.filename,
            id: version_id,
            url: file.url,
            sha256: file.hashes.sha512,
        })
    }

    async fn download<T>(&self, version_id: T) -> super::Result<TempFile>
    where
        T: Into<String>,
    {
        let tmp_file = TempFile::with_suffix(".jar")?;

        let file = self.fetch(version_id).await?;

        let response = self.0.get(file.url).send().await?;

        // Check ratelimit
        let ratelimit = modrinth_ratelimit(&response);

        match ratelimit {
            Ok(ratelimit) => {
                if ratelimit.is_reached() {
                    debug!("Reached ratelimit {:?}", ratelimit);
                    return Err(FetcherError::RatelimitReached);
                }
            }
            Err(e) => {
                if let FetcherError::NoRatelimit = e {
                    debug!("No ratelimit found.")
                }
            }
        }

        let content = response.bytes().await?;

        debug!("Fetched mod file sized {}.", content.len());

        fs::write(tmp_file.path(), content)?;

        Ok(tmp_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch() {
        let file = ModrinthFetcher::try_new()
            .unwrap()
            .download("k2tOj88k")
            .await
            .unwrap();

        file.cleanup().unwrap();
    }
}
