use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use russh_sftp::client::SftpSession;
use tokio::io::AsyncWriteExt;

use crate::{config::UpdaterConfig, ftp, NMUClient};

pub fn generate(nmu: &NMUClient) -> Result<(), &str> {
    if let Some(folder) = &nmu.work_folder {
        return generate_at(folder);
    } else if !nmu.ftp_location.address.is_empty() {
        return ftp::generate_over_sftp(nmu.ftp_location.clone());
    }
    Err("No work location set!")
}
pub fn generate_at(path: &PathBuf) -> Result<(), &str> {
    let mod_dir = path.join("mods");
    let mut vec: Vec<PathBuf> = Vec::new();
    match fs::read_dir(mod_dir) {
        Ok(files) => {
            for file_result in files {
                let path_buf = file_result.expect("Now this is not happening").path();
                vec.push(
                    path_buf
                        .strip_prefix(path)
                        .expect("Should be unreachable")
                        .to_path_buf(),
                );
            }
            if let Ok(mut file) = File::create(path.join(PathBuf::from("updater.json"))) {
                let config = UpdaterConfig {
                    files: vec,
                    pack_endpoint: None,
                };
                let json: String =
                    serde_json::to_string_pretty(&config).expect("Malformed struct somehow");
                file.write_all(json.as_bytes())
                    .expect("Could create but could not write to a file. Curious.");
                return Ok(());
            }
            Err("Could not create config file")
        }
        Err(_) => Err("Could not read mod directory!"),
    }
}
pub async fn generate_at_remote(ftp: &mut SftpSession) -> Result<(), &'static str> {
    let mut vec: Vec<PathBuf> = Vec::new();
    match ftp.read_dir("mods").await {
        Ok(files) => {
            for file_result in files {
                let path : PathBuf = ["mods", file_result.file_name().as_str()].iter().collect();
                vec.push(
                    path
                );
            }
            let config = UpdaterConfig {
                files: vec,
                pack_endpoint: None,
            };
            let json: String =
                serde_json::to_string_pretty(&config).expect("Malformed struct somehow");
            ftp.create("updater.json").await.unwrap().write(json.as_bytes()).await.unwrap();
            Ok(())

        }
        Err(_) => Err("Could not read mod directory!")
    }
}
