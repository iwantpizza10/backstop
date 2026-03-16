use std::{fs, io};

pub mod cache;
pub mod settings;
pub mod constants;
pub mod queue;

pub fn result_false_or_err<T>(result: Result<bool, T>) -> bool {
    match result {
        Err(_) => true,
        Ok(value) => !value
    }
}

pub fn init_config_dirs() -> Result<(), io::Error> {
    let dirs: [Option<String>; 2] = [
        {
            let dir = constants::conf_dir();

            if let Some(dir) = dir.to_str() {
                Some(dir.to_string())
            } else {
                None
            }
        },
        {
            let mut dir = constants::conf_dir();
            dir.push("covers");

            if let Some(dir) = dir.to_str() {
                Some(dir.to_string())
            } else {
                None
            }
        }
    ];

    for dir in dirs {
        if let Some(dir) = dir && result_false_or_err(fs::exists(&dir)) {
            // create_dir_all just in case theres somehow a missing folder on a higher level
            fs::create_dir_all(&dir)?;
        }
    }

    Ok(())
}
