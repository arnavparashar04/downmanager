mod cli;
mod downloader;
mod error;
mod headers;
mod http;
fn main() {
    let args : Vec<String>  = std::env::args().collect();
    println!("{:?}", args);
}
