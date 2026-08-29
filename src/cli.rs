use std::path::PathBuf;
use crate::{downloader::{self, DownloadStatus}, error::Error};
use std::io::{self, Write};
use std::collections::VecDeque;
use std::time::Instant;

pub struct Arguments{
    pub url : String,
    pub dest_path : Option<PathBuf>,
    pub help : bool,
    pub version : bool,
    pub recover : bool,
    pub force_connection : bool,
    pub force_connections_no : Option<u32>
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
                    recover: false,
                    force_connection: false,
                    force_connections_no: None
                });
            }

            "--version" | "-v" => {
                return Ok(Arguments {
                    url: String::new(),
                    dest_path: None,
                    help: false,
                    version: true,
                    recover: false,
                    force_connection: false,
                    force_connections_no: None
                });
            }
            "--recover" | "-r" => {
                return Ok(Arguments{
                    url : String::new(),
                    dest_path: None,
                    help: false,
                    version: false,
                    recover: true,
                    force_connection: false,
                    force_connections_no: None
                }
                )
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
            recover: false,
            force_connection: false,
            force_connections_no: None

        }),

        3 => Ok(Arguments {
            url: args[1].clone(),
            dest_path: Some(PathBuf::from(&args[2])),
            help: false,
            version: false,
            recover: false,
            force_connection: false,
            force_connections_no: None
        }),
        4 => Ok(Arguments {
            url: args[1].clone(),
            dest_path: None,
            help: false,
            version: false,
            recover: false,
            force_connection: true,
            force_connections_no: Some(args[3].parse().unwrap_or(1)as u32)

        }),



        _ => Err(Error::InvalidArguments),
    }
}

pub async fn display_progress(mut reciever : tokio::sync::watch::Receiver<downloader::DownloadProgress>){
    
    let mut progress_bar_index : usize = 0;
    let mut progress_bar = String::from("--------------------------------------------------");
    let mut progress_percent :f64 = 0.0; 
    const YELLOW: &str = "\x1b[33m";
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const RESET: &str = "\x1b[0m";
    let mut first_print = true;
    let mut speed_history: VecDeque<(Instant, u64)> = VecDeque::new();

    loop{
        if reciever.changed().await.is_err(){
            break;
        }
        let progress = reciever.borrow_and_update();
        let now = Instant::now();
        speed_history.push_back((now, progress.downloaded_size));
        while let Some((time, _)) = speed_history.front() {
            if now.duration_since(*time).as_secs() > 10 {
                speed_history.pop_front();
            } else{
                  break;
            }
        }
        let (mut down_size_formatted, mut down_postfix_format) = format_bytes(progress.downloaded_size);
        let (mut total_size_formatted, mut total_postfix_format) = (0.0, "");
        let (old_time, old_downloaded) = speed_history.front().unwrap();
        let elapsed = now.duration_since(*old_time).as_secs_f64();
        let speed = if elapsed > 0.0 {
            (progress.downloaded_size - *old_downloaded) as f64 / elapsed
        } else {
              0.0
        };
        let mut estimated_time = String::new();
        if let Some(total_size) = progress.total_size {
            estimated_time = calctimeleft(progress.downloaded_size, total_size, speed);
            (total_size_formatted, total_postfix_format) = format_bytes(total_size);
            progress_percent = (progress.downloaded_size*100) as f64/total_size as f64
        }
        let (mut speed_formatted, mut speed_postfix_format) = format_bytes(speed as u64);
        let target_index = progress_percent as usize / 2;
        if target_index != progress_bar_index {
            while progress_bar_index <target_index{
                progress_bar.replace_range(progress_bar_index..progress_bar_index + 1, "#"); 
                progress_bar_index+=1;
            }
        }
        if !first_print{
            print!("\x1b[3A");
        }
        else {
            first_print =  false;
        }
        print!("\x1b[2K\r"); //for clearing current line
        match progress.status {
            downloader::DownloadStatus::Connecting => print!("DOWNLOAD STATUS: {}CONNECTING{}\t", YELLOW, RESET), 
            downloader::DownloadStatus::Downloading => print!("DOWNLOAD STATUS: {}DOWNLOADING{}\t", GREEN, RESET), 
            downloader::DownloadStatus::ConnectingNewConnection => print!("DOWNLOAD STATUS: {}ADDING NEW CONNECTION{}\t", YELLOW, RESET),
            downloader::DownloadStatus::Stalled => print!("DOWNLOAD STATUS: {}STALLED{}\t", RED, RESET),
            downloader::DownloadStatus::Failed => print!("DOWNLOAD STATUS: {}FAILED{}\t", RED, RESET),
            downloader::DownloadStatus::Completed => print!("DOWNLOAD STATUS: {}COMPLETED{}\t", GREEN, RESET), 
        }
        print!("CONNECTIONS: {}\t", progress.connections);
        print!("HTTP STATUS: {:?}\n", progress.http_status.map(|s| s.as_u16()));
        print!("\x1b[2K\r");
        println!("{}  {:.1}%", progress_bar, progress_percent);
        print!("\x1b[2K\r");
        println!("TOTAL: {:.3} {}| SPEED: {:.2} {}/s | TIME LEFT: {}", total_size_formatted, total_postfix_format, speed_formatted, speed_postfix_format, estimated_time);

        io::stdout().flush().unwrap();
        if progress.status == downloader::DownloadStatus::Completed || progress.status == downloader::DownloadStatus::Failed{
            break;
        }
    }
}

fn format_bytes(input: u64) -> (f64, &'static str) {
    match input{
        x if x >= 1000000000000 => return (x as f64/1000000000000.0, "TB"),
        x if x >= 1000000000 => return (x as f64/1000000000.0, "GB"),
        x if x >= 1000000 => return (x as f64/1000000.0, "MB"),
        x if x >= 1000 => return (x as f64/1000.0, "KB"),
        _ => return(input as f64, "B"),
    }    
}

fn calctimeleft(downloaded: u64, total: u64, speed: f64) -> String{
    let mut time_left :u64 =0;
    if speed != 0.0{
        time_left = (total - downloaded)/ speed as u64;
    }
    else{
        return format!("Unknown");
    }
    let hours = time_left/3600;
    let minutes = (time_left % 3600) / 60;
    let secs = time_left % 60;
    
    if hours == 0 && minutes==0{
        return format!("{}s", secs);
    }
    else if hours == 0 {
        return format!("{}m:{}s", minutes,secs);    
    }
    else{
        return format!("{}h:{}m:{}s", hours, minutes, secs);
    }
}
