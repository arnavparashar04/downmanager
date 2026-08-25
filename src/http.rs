use crate::error::Error;

pub async fn get(url: &str) -> Result<reqwest::Response, Error>{
    let response = reqwest::get(url).await.map_err(Error::Network)?;
    Ok(response)
}    
