mod cli;
mod downloader;
mod error;
mod http;
use std::path::Path; 

const VERSION : &str = "1.4.0";

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let args : Vec<String>  = std::env::args().collect();
    let parsed_args : cli::Arguments = cli::parse_args(&args)?;
    if parsed_args.version {
        println!("{}", VERSION);
    }

    if !parsed_args.url.is_empty(){
        let progress = downloader::DownloadProgress::new();
        let (transmitter,reciever) = tokio::sync::watch::channel(progress);
        let (downloadresult, _) = tokio::join!(downloader::download(&parsed_args.url, Path::new("test"), transmitter), cli::display_progress(reciever));
        downloadresult?;
    } 
    Ok(())
}    
