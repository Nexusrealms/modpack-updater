use std::{
    fs::{self, File},
    io::{Cursor, Write},
    path::PathBuf,
};

use crate::{config::UpdaterConfig, NMUClient};

pub fn generate(client: &NMUClient) -> Result<(), &str> {
    if let Some(work_folder) = &client.work_folder {
        return generate_at(work_folder);
    }
    Result::Err("The work folder needs to be set!")
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
/*pub fn generate_at_remote(ftp: &mut SftpStream) -> Result<(), &'static str> {
    let mut vec: Vec<PathBuf> = Vec::new();
    match ftp.list(Some("mods")) {
        Ok(files) => {
            for file_result in files {
                let path : PathBuf = ["mods", file_result.as_str()].iter().collect();
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
            ftp.put_file("updater.json", &mut Cursor::new(json.as_bytes())).unwrap();
            Ok(())
        }
        Err(_) => Err("Could not read mod directory!")
    }
}*/
