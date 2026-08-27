use futures_util::StreamExt;
use std::io::Write;
use crate::http;
use std::time::Instant;
use crate::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct DownloadProgress{
    pub status : DownloadStatus,
    pub http_status : Option<reqwest::StatusCode>,
    pub connections : u8,
    pub total_size : Option<u64>, //because we dont always get total size from the request
    pub downloaded_size : u64,
    pub started_at: Option<Instant>,
    pub last_bytestream: Option<Instant> 
}
#[derive(Debug, Clone, PartialEq)]
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
            http_status: None, 
            connections: 0,
            total_size: None,
            downloaded_size: 0,
            started_at: None,
            last_bytestream: None,
        }
    }
}

fn init_file(dest: &Path) -> Result<std::fs::File, Error>{
    let mut count = 0;
    let mut path = dest.to_path_buf();
    loop{
        match std::fs::File::create_new(&path) {
            Ok(file) => return Ok(file),
            Err(e) if e.kind() ==std::io::ErrorKind::AlreadyExists => {
                count +=1;
                let name = dest.file_stem().and_then(|s| s.to_str()).unwrap_or("Untitled");
                let ext = dest.extension().and_then(|s| s.to_str());
                let filename = match ext {
                    Some(ext) => format!("{} ({}).{}", name, count, ext),
                    None => format!("{} ({})", name, count),
                };
                path = dest.with_file_name(filename);
            },
            Err(e) => return Err(Error::File(e)),
        }
    }
}
pub async fn download(url: &str, transmitter: tokio::sync::watch::Sender<DownloadProgress>) -> Result<(), Error>{
    let mut progress = DownloadProgress::new();
    transmitter.send(progress.clone()).map_err(|_| Error::Channel)?; //for connecting state
    let response = http::get(url).await?;
    progress.http_status = Some(response.status());
    progress.total_size = response.content_length(); 
    if !response.status().is_success(){
        return Err(Error::NetworkStatus(response.status()))
    }
    progress.connections = 1;
    transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
    let out = get_filename(&response, url)?;
    let mut fileout = init_file(&out)?;
    download_stream(response, &mut fileout, &mut progress, &transmitter).await?; 
    Ok(())

}

async fn download_stream(response: reqwest::Response, fileout: &mut std::fs::File, progress: &mut DownloadProgress, transmitter: &tokio::sync::watch::Sender<DownloadProgress>) -> Result< (), Error>{ 
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
        transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
    }
    progress.status = DownloadStatus::Completed;
    progress.connections = 0;
    transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
    Ok(())
}

fn get_filename(response: &reqwest::Response, url: &str) -> Result<PathBuf,Error>{
    let content_diposition = response.headers().get(reqwest::header::CONTENT_DISPOSITION).and_then(|value| value.to_str().ok());
    let filename = content_diposition.and_then(|value| value.split("filename=").nth(1)).map(|name| name.trim_matches('"'));
    match filename {
       Some(name) => Ok(PathBuf::from(name)),
       None =>{
           let url1 = reqwest::Url::parse(url).map_err(|_| Error::InvalidArguments)?; //other method
           let filename1 = url1.path_segments().and_then(|segments| segments.last()).unwrap_or("Untitled");
           Ok(PathBuf::from(filename1))
       }
    }    
}
