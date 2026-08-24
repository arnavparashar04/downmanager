use std::path::PathBuf;
use crate::error::Error;

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
