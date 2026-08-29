mod cli;
mod downloader;
mod error;
mod http;


use reqwest::Url;

use crate::error::Error; 

const VERSION : &str = "1.8.5";

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let args : Vec<String>  = std::env::args().collect();
    let parsed_args : cli::Arguments = cli::parse_args(&args)?;
    if parsed_args.version {
        println!("{}", VERSION);
    }
    if parsed_args.help{
        println!("Download Manager\n--version -> Print version\n--help -> Print this help message \n--recover -> Look for recoverable files and start recovering the download\ndownmanager <URL> -> To download the file in the url");
    }

    if !parsed_args.url.is_empty(){
        let progress = downloader::DownloadProgress::new();
        let (transmitter,reciever) = tokio::sync::watch::channel(progress);
        let (downloadresult, _) = tokio::join!(downloader::download(&parsed_args.url,transmitter, &parsed_args.force_connection, &parsed_args.force_connections_no), cli::display_progress(reciever));
        downloadresult?;
    }
    print!("\n");
    Ok(())
}


