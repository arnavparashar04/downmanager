use std::path::Path;
use futures_util::StreamExt; //for stream nd next()
use std::io::Write;
use crate::http;

use crate::error::Error;

fn init_file(dest: &Path) -> Result<std::fs::File, Error>{
    let fileout = std::fs::File::create_new(dest).map_err(Error::File)?;
    Ok(fileout)
}
pub async fn download(url: &str, out : &Path) -> Result<(), Error>{
    let response = http::get(url).await?;
    if !response.status().is_success(){
        return Err(Error::NetworkStatus(response.status()))
    }
    let mut fileout = init_file(out)?;
    download_stream(response, &mut fileout).await?; 
    Ok(())

}

async fn download_stream(response: reqwest::Response, fileout: &mut std::fs::File) -> Result< (), Error>{ 
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await{
        let chunk = chunk.map_err(Error::Network)?; //since chunk is still a result from reqwest
        fileout.write_all(&chunk).map_err(Error::File)?;
    }
    Ok(())
}
