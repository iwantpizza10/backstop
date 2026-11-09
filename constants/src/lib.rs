use std::{fs, io, path::PathBuf};

pub fn conf_dir() -> PathBuf {
    let mut dir = dirs::config_local_dir().expect("config dir should return. if you're seeing this, you're likely running on a non-supported platform");

    // todo for later: make this not be dev ig idk like ???
    dir.push("backstop_dev");

    dir
}

/// call this once upon app init
pub fn init_everything() -> Result<(), io::Error> {
    let mut dir = conf_dir();
    fs::create_dir_all(&dir)?;
    dir.push("covers");
    fs::create_dir_all(&dir)?;

    Ok(())
}
