use crate::fetcher::FetcherError;
use log::trace;
use reqwest::Response;

#[derive(Debug)]
pub struct Ratelimit {
    limit: usize,
    remaining: usize,
    reset: usize,
}

impl Ratelimit {
    pub fn is_reached(&self) -> bool {
        self.remaining == 0
    }
}

pub fn modrinth_ratelimit(response: &Response) -> Result<Ratelimit, FetcherError> {
    let f = |key: &str| {
        trace!("Extract key {} from header.", key);

        for (key, value) in response.headers() {
            trace!("Header: {:20}->\t{:20}", key, value.to_str().unwrap())
        }

        Ok::<usize, FetcherError>(
            response
                .headers()
                .get(key)
                .ok_or(FetcherError::NoRatelimit)?
                .to_str()
                .unwrap()
                .parse::<usize>()
                .unwrap(),
        )
    };

    Ok(Ratelimit {
        limit: f("X-Ratelimit-Limit")?,
        remaining: f("X-Ratelimit-Remaining")?,
        reset: f("X-Ratelimit-Reset")?,
    })
}
