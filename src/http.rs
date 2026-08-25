use crate::error::Error;

pub async fn get(url: &str) -> Result<(), Error>{
    let response = reqwest::get(url).await.map_err(Error::Network)?;
    println!("Status : {}", response.status());
    println!("Headers : {:#?}", response.headers());
    Ok(())
}    
