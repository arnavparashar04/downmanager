use std::path::Path;
use futures_util::StreamExt; //for stream nd next()
use std::io::Write;

use crate::error::Error;

fn init_file(dest: &Path) -> Result<std::fs::File, Error>{
    let fileout = std::fs::File::create_new(dest).map_err(Error::File)?;
    Ok(fileout)
}
pub async fn download(url: &str, out : &Path) -> Result<(), Error>{
        
    Ok(())
}

async fn download_stream(response: reqwest::Response, fileout: &mut std::fs::File) -> Result< (), Box<dyn std::error::Error>>{ //box dyn error just for now later will switch to my Error
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await{
        let chunk = chunk?; //since chunk is still a result from reqwest, this unwraps it to &[u8]
        fileout.write_all(&chunk)?;
    }
    Ok(())
}
