use std::path::Path;

pub fn recoveryexists() -> bool {
    Path::new("recover.downmanager").is_file()
}
