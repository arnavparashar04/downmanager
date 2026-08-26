mod cli;
mod downloader;
mod error;
mod http;
use std::path::Path; 

const VERSION : &str = "1.0.0";

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let args : Vec<String>  = std::env::args().collect();
    let parsed_args : cli::Arguments = cli::parse_args(&args)?;
    if parsed_args.version {
        println!("{}", VERSION);
    }
    if !parsed_args.url.is_empty(){
        downloader::download(&parsed_args.url, Path::new("test")).await?;
    } 
    Ok(())
}    
