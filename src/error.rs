#[derive(Debug)]
pub enum Error {
   InvalidArguments,
   InvalidFilePath,
   InsufficientSpace,
   Network(reqwest::Error),
   InvalidUrl,
   UnsupportedDownloadUrl(String),
   TempNotFound,
   TerminalSize
}

