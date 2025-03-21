use std::{
    fs::{self, File}, io::Write, path::{Path, PathBuf}
};
const CONFIG_NAME: &str = "updater.json";
use serde::{Deserialize, Serialize};

pub fn load_config(path: &Path) -> Result<UpdaterConfig, &str> {
    let file = fs::read_to_string(path.join(Path::new(CONFIG_NAME)));
    let result: Option<UpdaterConfig> = match file {
        Ok(string) => serde_json::from_str(string.as_str()).ok(),
        Err(_e) => None,
    };
    result.ok_or("Could not get updater config!")
}
pub fn write_config(path: &Path, config: &UpdaterConfig) -> Result<(), &'static str> {
    if let Ok(mut file) =  File::create(path.join(Path::new(CONFIG_NAME))){
        if let Ok(content) = serde_json::to_string_pretty(config){
            return match file.write_all(content.as_bytes()) {
                Ok(_) => Ok(()),
                Err(_) => Err("Could not write JSON to file")
            };
        }
        return Err("Could not serialize config");
    }
    Err("Could not create config file")
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdaterConfig {
    pub(crate) files: Vec<PathBuf>,
    pub(crate) pack_endpoint: Option<String>,
}
pub fn delete_by_config(path: &Path, config: &UpdaterConfig) -> Result<(), &'static str> {
    for file in &config.files {
        match fs::remove_file(path.join(file)) {
            Ok(_) => {},
            Err(_) => {}
        }
    }
    Ok(())
}
