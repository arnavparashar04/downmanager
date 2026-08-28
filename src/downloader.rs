use futures_util::{StreamExt};
use tokio::io::AsyncWriteExt;
use std::os::unix::fs::FileExt;
use crate::http;
use std::time::Instant;
use crate::error::Error;
use std::path::{Path, PathBuf};
use std::sync::Arc;

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

async fn init_file(dest: &Path) -> Result<tokio::fs::File, Error>{
    let mut count = 0;
    let mut path = dest.to_path_buf();
    loop{
        match tokio::fs::OpenOptions::new().write(true).create_new(true).open(&path).await {
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
    let (response,range_supportcheck2) = http::get(url).await?;
    progress.http_status = Some(response.status());
    progress.total_size = response.content_length(); 
    if !response.status().is_success(){
        return Err(Error::NetworkStatus(response.status()))
    }
    let out = get_filename(&response, url)?;
    let mut fileout = init_file(&out).await?;
    let supports_ranges = response.headers().get(reqwest::header::ACCEPT_RANGES).and_then(|value| value.to_str().ok()).map(|value| value.eq_ignore_ascii_case("bytes")).unwrap_or(false);
    match (progress.total_size, supports_ranges, range_supportcheck2){
        
        (Some(total_size), true, true) => {
            
            fileout.set_len(total_size).await.map_err(Error::File)?;
            let fileout = fileout.into_std().await;
            let fileout = Arc::new(fileout);
            let connections = split_connections(total_size, 4);//4 just for now
            progress.connections = connections.len() as u8;
            transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
            let client = reqwest::Client::new();
            let (progress_transmitter, mut progress_reciever) = tokio::sync::mpsc::channel(100);
            let mut tasks = Vec::new(); //basically for futures returned by the download connection
            
            for connection in connections{
                let progress_transmitter = progress_transmitter.clone();
                let file = Arc::clone(&fileout);
                tasks.push(download_connection(url, &client, connection, total_size,file, progress_transmitter));

            }
            drop(progress_transmitter);
            let downloaded_tasks = futures_util::future::try_join_all(tasks);
            tokio::pin!(downloaded_tasks); //pins in stack
            loop{
                tokio::select! {
                    result = &mut downloaded_tasks =>{
                        result?;
                        break;
                    }
                    Some(bytes) = progress_reciever.recv() => {
                        progress.downloaded_size +=bytes;
                        progress.last_bytestream = Some(Instant::now());
                        if progress.started_at.is_none() {
                           progress.started_at = Some(Instant::now());
                           progress.status = DownloadStatus::Downloading;
                        }
                        transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
                    }
                }
            }
            progress.downloaded_size = total_size;
            progress.connections = 0;
            progress.status = DownloadStatus::Completed;
            transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
        },
        _ => {
            progress.connections = 1;
            transmitter.send(progress.clone()).map_err(|_| Error::Channel)?;
            download_stream(response, &mut fileout, &mut progress, &transmitter).await?; 
        }
    }
    
    Ok(())

}

async fn download_stream(response: reqwest::Response, fileout: &mut tokio::fs::File, progress: &mut DownloadProgress, transmitter: &tokio::sync::watch::Sender<DownloadProgress>) -> Result< (), Error>{ 
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await{
        let chunk = chunk.map_err(Error::Network)?; //since chunk is still type result<> from reqwest
        fileout.write_all(&chunk).await.map_err(Error::File)?;
        
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

#[derive(Debug, Clone, Copy)]
pub struct ByteRange{
    start: u64,
    end: u64
}

impl ByteRange{
    fn size(&self) -> u64{
        self.end - self.start + 1
    }
}

pub struct Connection{
    pub id :usize,
    pub range: ByteRange,
    pub downloaded: u64
}

fn split_connections(total_size: u64, connectionsno: usize) ->Vec<Connection>{
    let split_size = total_size/connectionsno as u64;
    let remainder = total_size%connectionsno as u64;
    let mut connections = Vec::with_capacity(connectionsno);
    let mut start = 0;

    for id in 0..connectionsno{
        let size = if id==connectionsno - 1{
            split_size + remainder
        }
        else {
            split_size
        };
        let end = start + size - 1;
        connections.push(Connection{id, range : ByteRange{start, end}, downloaded : 0});
        start = end + 1;
    }
    connections
}

async fn download_connection(url: &str, client: &reqwest::Client, mut connection: Connection, total_size: u64, file: Arc<std::fs::File>, progress_transmitter: tokio::sync::mpsc::Sender<u64>) -> Result<(), Error> {
    const BATCH_SIZE: usize = 1024 * 1024;
    let response = http::getrange(url, client, connection.range.start, connection.range.end).await?;
    let content_range = response.headers().get(reqwest::header::CONTENT_RANGE).ok_or(Error::InvalidArguments)?;
    let content_range = content_range.to_str().map_err(|_| Error::InvalidArguments)?;
    let (response_start, response_end, response_total) = parse_content_range(content_range)?;
    if response_start != connection.range.start || response_end != connection.range.end || response_total != total_size {
        return Err(Error::InvalidArguments);
    }
    let mut stream = response.bytes_stream();
    let mut offset = connection.range.start;
    let mut buffer: Vec<u8> = Vec::with_capacity(BATCH_SIZE);

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(Error::Network)?;
        buffer.extend_from_slice(&chunk);

        if buffer.len() >= BATCH_SIZE {
            let file = Arc::clone(&file);
            let bytes = std::mem::take(&mut buffer);
            let bytes_len = bytes.len();
            tokio::task::spawn_blocking(move || file.write_all_at(&bytes, offset)).await.map_err(|_| Error::File(std::io::Error::other("blocking write task failed")))?.map_err(Error::File)?;
            offset += bytes_len as u64;
            connection.downloaded += bytes_len as u64;
            progress_transmitter.send(bytes_len as u64).await.map_err(|_| Error::Channel)?;
        }
    }
    if !buffer.is_empty() {
        let file = Arc::clone(&file);
        let bytes = std::mem::take(&mut buffer);
        let bytes_len = bytes.len();
        tokio::task::spawn_blocking(move || file.write_all_at(&bytes, offset)).await.map_err(|_| Error::File(std::io::Error::other("blocking write task failed")))?.map_err(Error::File)?;
        connection.downloaded += bytes_len as u64;
        progress_transmitter.send(bytes_len as u64).await.map_err(|_| Error::Channel)?;
    }

    if connection.downloaded != connection.range.size() {
        return Err(Error::InvalidArguments);
    }
    Ok(())
}

fn parse_content_range(value: &str) -> Result<(u64, u64, u64), Error> {
    let value = value.strip_prefix("bytes ").ok_or(Error::InvalidArguments)?;
    let (range, total) = value.split_once('/').ok_or(Error::InvalidArguments)?;
    let (start, end) = range.split_once('-').ok_or(Error::InvalidArguments)?;
    let start = start.parse::<u64>().map_err(|_| Error::InvalidArguments)?;
    let end = end.parse::<u64>().map_err(|_| Error::InvalidArguments)?;
    let total = total.parse::<u64>().map_err(|_| Error::InvalidArguments)?;
    Ok((start, end, total))
}
