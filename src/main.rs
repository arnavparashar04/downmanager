mod cli;
mod downloader;
mod error;
mod headers;
mod http;
fn main() -> Result<(), error::Error> {
    let args : Vec<String>  = std::env::args().collect();
    let parsed_args : cli::Arguments = cli::parse_args(&args)?;
    println!("{}", parsed_args.url);
    Ok(())
}    
