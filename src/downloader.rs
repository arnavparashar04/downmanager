use std::path::Path;

use crate::error::Error;

fn init_file(dest: &Path) -> Result<std::fs::File, Error>{
    let fileout = std::fs::File::create_new(dest).map_err(Error::File)?;
    Ok(fileout)
}
pub async fn download(url: &str, out : &Path) -> Result<(), Error>{
    
    Ok(())
}

async fn download_stream(response: reqwest::Response, fileout: &mut std::fs::File) -> Result< (), Error>{
    
    Ok(())
}
