use crate::error::Error;

pub async fn get(url: &str) -> Result<(reqwest::Response, bool), Error>{
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.map_err(Error::Network)?;
    let range_response = client.get(url).header(reqwest::header::RANGE, "bytes=0-0").send().await.map_err(Error::Network)?;
    let supports_partial =range_response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    Ok((response, supports_partial))
}

use tokio::time::{sleep, Duration};

pub async fn getrange(url: &str,client: &reqwest::Client,start: u64,end: u64,) -> Result<reqwest::Response, Error> {
    let mut delay = Duration::from_secs(1);
    for _ in 0..5 {
        let response = client
            .get(url)
            .header(
                reqwest::header::RANGE,
                format!("bytes={}-{}", start, end),
            )
            .send()
            .await
            .map_err(Error::Network)?;
        match response.status() {
            reqwest::StatusCode::PARTIAL_CONTENT => {
                return Ok(response);
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                sleep(delay).await;
                delay *= 2;
            }
            reqwest::StatusCode::OK => {
                return Err(Error::NoRangeSupport);
            }
            status => {
                return Err(Error::NetworkStatus(status));
            }
        }
    }
    Err(Error::RateLimited)
}
