use std::path::Path;

use reqwest::Error;

pub fn recoveryexists() -> bool {
    Path::new("recover.downmanager").is_file()
}

pub fn writerecoveryinfo() -> Result<(), Error>{
    
    Ok(())
}
