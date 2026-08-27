use crate::error::Error;

pub async fn get(url: &str) -> Result<reqwest::Response, Error>{
    let response = reqwest::get(url).await.map_err(Error::Network)?;
    Ok(response)
}

pub async fn getrange(url: &str, client : &reqwest::Client, start : u64, end : u64) -> Result<reqwest::Response, Error>{
    let response = client.get(url).header(reqwest::header::RANGE, format!("bytes={}-{}", start, end)).send().await.map_err(Error::Network)?;
    if response.status() != reqwest::StatusCode::PARTIAL_CONTENT{
        return Err(Error::NoRangeSupport)
    }
    Ok(response)
}

