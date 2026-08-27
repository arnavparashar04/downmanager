use std::io;

#[derive(Debug)]
pub enum Error {
   InvalidArguments,
   InvalidFilePath,
   InsufficientSpace,
   Network(reqwest::Error),
   InvalidUrl,
   UnsupportedDownloadUrl(String),
   TempNotFound,
   TerminalSize,
   File(std::io::Error),
   NetworkStatus(reqwest::StatusCode),
   Channel,
   NoRangeSupport
}

