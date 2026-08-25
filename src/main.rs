mod cli;
mod downloader;
mod error;
mod http;

const VERSION : &str = "0.0.1";

#[tokio::main]
async fn main() -> Result<(), error::Error> {
    let args : Vec<String>  = std::env::args().collect();
    let parsed_args : cli::Arguments = cli::parse_args(&args)?;
    if parsed_args.version {
        println!("{}", VERSION);
    }
    if !parsed_args.url.is_empty(){
        http::get(&parsed_args.url).await?;
    } 
    Ok(())
}    
