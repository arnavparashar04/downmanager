use std::path::PathBuf;
use crate::{downloader::{self, DownloadStatus}, error::Error};
use std::io::{self, Write};

pub struct Arguments{
    pub url : String,
    pub dest_path : Option<PathBuf>,
    pub help : bool,
    pub version : bool,
}

pub fn parse_args(args: &[String]) -> Result<Arguments, Error> {
    if args.len() == 2 {
        match args[1].as_str() {
            "--help" | "-h" => {
                return Ok(Arguments {
                    url: String::new(),
                    dest_path: None,
                    help: true,
                    version: false,
                });
            }

            "--version" | "-v" => {
                return Ok(Arguments {
                    url: String::new(),
                    dest_path: None,
                    help: false,
                    version: true,
                });
            }

            _ => {}
        }
    }

    match args.len() {
        2 => Ok(Arguments {
            url: args[1].clone(),
            dest_path: None,
            help: false,
            version: false,
        }),

        3 => Ok(Arguments {
            url: args[1].clone(),
            dest_path: Some(PathBuf::from(&args[2])),
            help: false,
            version: false,
        }),

        _ => Err(Error::InvalidArguments),
    }
}

pub async fn display_progress(mut reciever : tokio::sync::watch::Receiver<downloader::DownloadProgress>){
    loop{
        if reciever.changed().await.is_err(){
            break;
        }
        let progress = reciever.borrow_and_update();
        print!("\rDownloaded: {} | Status: {:?}",progress.downloaded_size,progress.status);
        io::stdout().flush().unwrap();
        if progress.status == downloader::DownloadStatus::Completed || progress.status == downloader::DownloadStatus::Failed{
            break;
        }
    }
}
