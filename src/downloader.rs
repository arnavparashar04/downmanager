use std::path::Path;
use futures_util::StreamExt;
use std::io::Write;
use crate::http;
use std::time::Instant;
use crate::error::Error;

#[derive(Debug)]
pub struct DownloadProgress{
    pub status : DownloadStatus,
    pub http_status : Option<reqwest::StatusCode>,
    pub connections : u8,
    pub total_size : Option<u64>, //because we dont always get total size from the request
    pub downloaded_size : u64,
    pub started_at: Option<Instant>,
    pub last_bytestream: Option<Instant> 
}
#[derive(Debug)]
pub enum DownloadStatus{
    Connecting,
    Downloading,
    ConnectingNewConnection,
    Stalled,
    Completed,
    Failed,
}

impl DownloadProgress{
    pub fn new() -> Self{
        Self{
            status: DownloadStatus::Connecting,
            http_status: None, // temporary
            connections: 0,
            total_size: None,
            downloaded_size: 0,
            started_at: None,
            last_bytestream: None,
        }
    }
}

fn init_file(dest: &Path) -> Result<std::fs::File, Error>{
    let fileout = std::fs::File::create_new(dest).map_err(Error::File)?;
    Ok(fileout)
}
pub async fn download(url: &str, out : &Path, progress: &mut DownloadProgress) -> Result<(), Error>{
    let response = http::get(url).await?;
    progress.http_status = Some(response.status());
    progress.total_size = response.content_length();
    progress.connections = 1; 
    if !response.status().is_success(){
        return Err(Error::NetworkStatus(response.status()))
    }
    let mut fileout = init_file(out)?;
    download_stream(response, &mut fileout, progress).await?; 
    Ok(())

}

async fn download_stream(response: reqwest::Response, fileout: &mut std::fs::File, progress: &mut DownloadProgress) -> Result< (), Error>{ 
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await{
        let chunk = chunk.map_err(Error::Network)?; //since chunk is still type result<> from reqwest
        fileout.write_all(&chunk).map_err(Error::File)?;
        
        let thischunktime = std::time::Instant::now();
        progress.downloaded_size += chunk.len() as u64;
        progress.last_bytestream = Some(thischunktime);
        if progress.started_at.is_none(){
            progress.started_at = Some(thischunktime);
            progress.status = DownloadStatus::Downloading;
        }
    }
    Ok(())
}
