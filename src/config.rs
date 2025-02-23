use std::{fs, io, path::PathBuf};

use winsafe::{SHGetKnownFolderPath, co};

pub fn get_config_path() -> PathBuf {
    if let Ok(path) =
        SHGetKnownFolderPath(&co::KNOWNFOLDERID::LocalAppDataLow, co::KF::DEFAULT, None)
    {
        PathBuf::from(path)
    } else {
        PathBuf::from(".")
    }
    .join("gacha_url_last_game.txt")
}

pub fn save_last_game(index: u32) -> io::Result<()> {
    let path = get_config_path();
    fs::write(path, index.to_string())
}

pub fn load_last_game() -> Option<u32> {
    let path = get_config_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|s| s.trim().parse().ok())
}
