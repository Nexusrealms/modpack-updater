use std::path::Path;

use crate::{
    config::{delete_by_config, load_config, write_config},
    mrpack::update_from_mrpack_to_local,
    PackSource,
};

pub fn run_local(folder: &Path, source: &PackSource) -> Result<(), &'static str> {
    match source {
        PackSource::None => Err("No pack source set!"),
        _ => {
            let folder_path = folder;
            if let Ok(config) = load_config(folder_path) {
                match delete_by_config(folder_path, &config) {
                    Ok(_) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            };
            match update_from_mrpack_to_local(source, folder) {
                Ok(config) => write_config(folder, &config),
                Err(str) => Err(str),
            }
        }
    }
}
