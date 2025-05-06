use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

use russh_sftp::client::SftpSession;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncWriteExt, runtime::Runtime};

use crate::{config::UpdaterConfig, PackSource};
pub fn update_from_mrpack_to_local(
    source: &PackSource,
    work_folder: &Path,
) -> Result<UpdaterConfig, &'static str> {
    let rt = Runtime::new().unwrap();
    let pack = rt.block_on(get_mrpack(source));
    match pack {
        Ok((pack, url_option)) => match transfer_pack_files_to_local(pack, work_folder) {
            Ok(vec) => Ok(UpdaterConfig {
                files: vec,
                pack_endpoint: url_option,
            }),
            Err(str) => Err(str),
        },
        Err(str) => Err(str),
    }
}
pub async fn update_from_mrpack_to_remote(
    source: &PackSource,
    sftp: &mut SftpSession,
) -> Result<UpdaterConfig, &'static str> {
    match get_mrpack(source).await {
        Ok((pack, url_option)) => match transfer_pack_files_to_remote(pack, sftp).await {
            Ok(vec) => Ok(UpdaterConfig {
                files: vec,
                pack_endpoint: url_option,
            }),
            Err(str) => Err(str),
        },
        Err(str) => Err(str),
    }
}
pub async fn get_mrpack(source: &PackSource) -> Result<(Mrpack, Option<String>), &'static str> {
    match source {
        PackSource::FromFile(path) => {
            if let Ok(file) = fs::File::open(path) {
                if let Ok(mut zip) = zip::ZipArchive::new(file) {
                    if let Ok(mut pack_file) = zip.by_name("modrinth.index.json") {
                        let mut contents = String::new();
                        pack_file
                            .read_to_string(&mut contents)
                            .expect("Could not read file content ?");
                        return match serde_json::from_str::<Mrpack>(contents.as_str()) {
                            Ok(pack) => Result::Ok((pack, None)),
                            Err(_) => Result::Err("Could not deserialize pack file"),
                        };
                    }
                }
            }
            Result::Err("Could not open .mrpack file")
        }
        PackSource::Url(url) => {
            if let Ok(response) = reqwest::get(url).await {
                let mut tmpfile = tempfile::tempfile().expect("Could not create tempfile");
                tmpfile
                    .write_all(&response.bytes().await.unwrap())
                    .expect("Could not copy to tempfile");
                if let Ok(mut zip) = zip::ZipArchive::new(tmpfile) {
                    if let Ok(mut pack_file) = zip.by_name("modrinth.index.json") {
                        let mut contents = String::new();
                        pack_file
                            .read_to_string(&mut contents)
                            .expect("Could not read file content ?");
                        return match serde_json::from_str::<Mrpack>(contents.as_str()) {
                            Ok(pack) => Result::Ok((pack, Some(url.clone()))),
                            Err(_) => Result::Err("Could not deserialize pack file"),
                        };
                    }
                }
                return Err("Could not unzip downloaded mrpack");
            }
            Err("Could not GET mrpack file")
        }
        PackSource::None => Err("No pack source selected"),
    }
}
fn transfer_pack_files_to_local(pack: Mrpack, folder: &Path) -> Result<Vec<PathBuf>, &'static str> {
    let mut paths = Vec::new();
    for PackEntry { path, downloads } in pack.files {
        if !downloads.is_empty() {
            if let Ok(mut response) = reqwest::blocking::get(&downloads[0]) {
                if let Ok(mut file) = fs::File::create(folder.join(&path)) {
                    io::copy(&mut response, &mut file).expect("Could not write into created file");
                    paths.push(path);
                } else {
                    return Err("Could not create file in mod directory");
                }
            } else {
                return Err("Could not GET file from download link in pack definition");
            }
        } else {
            return Err("File has no download links");
        }
    }
    Ok(paths)
}
async fn transfer_pack_files_to_remote(
    pack: Mrpack,
    sftp: &mut SftpSession,
) -> Result<Vec<PathBuf>, &'static str> {
    let mut paths = Vec::new();
    for PackEntry { path, downloads } in pack.files {
        if !downloads.is_empty() {
            if let Ok(response) = reqwest::get(&downloads[0]).await {
                if let Ok(mut file) = sftp.create(path.to_string_lossy()).await {
                    file.write_all(&response.bytes().await.unwrap())
                        .await
                        .expect("Could not write into created file!");
                    paths.push(path);
                } else {
                    return Err("Could not create file in mod directory");
                }
            } else {
                return Err("Could not GET file from download link in pack definition");
            }
        } else {
            return Err("File has no download links");
        }
    }
    Ok(paths)
}
#[derive(Serialize, Deserialize, Debug)]
pub struct Mrpack {
    files: Vec<PackEntry>,
}
#[derive(Serialize, Deserialize, Debug)]
struct PackEntry {
    path: PathBuf,
    downloads: Vec<String>,
}
